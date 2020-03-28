extern crate csv;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
extern crate chrono;

use chrono::{NaiveDateTime, Utc};
use serde::{Serialize};

use std::time::{Duration, Instant};
use actix::prelude::*;
use uuid::Uuid;

use csv::{ReaderBuilder};
use serde::{Deserialize};
use actix_web::{post, get, put, delete, web, App, Error, middleware, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::{Arc, Mutex};
use std::io;
use actix_files as fs;
use actix_web_actors::ws;
use std::clone::Clone;

use std::cmp;

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatedProduct {
    pub name: Option<String>,
    pub price: Option<f64>, //Price should managed as an integer to avoid floating point errors
    pub barcode: Option<String>,
    pub department: Option<String>,
    pub supplier: Option<String>,
    #[serde(rename = "current_stock")]
    pub stock_level: Option<i64>,
    pub max_stock: Option<i64>,
    pub version: Option<i64>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewProduct {
    pub name: String,
    pub price: f64, //Price should managed as an integer to avoid floating point errors
    pub barcode: String,
    pub department: String,
    pub supplier: String,
    #[serde(rename = "current_stock")]
    pub stock_level: i64,
    pub max_stock: i64
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct Product {
    pub id: String, //if the product is new it won't have an ID
    pub name: String,
    pub price: i64, //Price is managed as an integer to avoid floating point errors
    pub barcode: String,
    pub department: String,
    pub supplier: String,
    pub label_printed: Option<bool>,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub deleted: bool,
    #[serde(rename = "current_stock")]
    pub stock_level: i64,
    pub max_stock: i64,
    pub version: i64,
}

impl Product {
    pub fn new(name: String, price: i64, barcode: String, department: String, supplier: String, stock_level: i64, max_stock: i64) -> Product {
        Product {
            id: Uuid::new_v4().to_string(),
            name: name,
            price: price,
            barcode: barcode,
            department: department,
            supplier: supplier,
            label_printed: Some(false),
            created: NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0),
            updated: NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0),
            deleted: false,
            stock_level: stock_level,
            max_stock: max_stock,
            version: 0
        }
    }
    pub fn from_new_product(new_product: NewProduct) -> Product {
        Product::new(new_product.name, (new_product.price/100.0) as i64, new_product.barcode, new_product.department, new_product.supplier, new_product.stock_level, new_product.max_stock)
    }
    pub fn delete(&mut self) {
        self.deleted = true;
        self.version += 1;
    }
    pub fn update_product(&mut self, updated_product: UpdatedProduct) -> Result<Vec<String>, &'static str> {
        if self.deleted {
            return Err("Product deleted");
        }
        let version = match updated_product.version {
            Some(version) => version,
            None => return Err("Version not specified")
        };
        if version != self.version {
            return Err("Out of date");
        }
        let mut updated_fields: Vec<String> = Vec::new();
        match updated_product.name {
            Some(name) => {
                self.name = name;
                updated_fields.push(String::from("name"));
            },
            None => {}
        };
        match updated_product.price {
            Some(price) => {
                self.price = (price / 100.00) as i64;
                updated_fields.push(String::from("price"));
            },
            None => {}
        };
        match updated_product.supplier {
            Some(supplier) => {
                self.supplier = supplier;
                updated_fields.push(String::from("price"));
            },
            None => {}
        };
        match updated_product.max_stock {
            Some(max_stock) => {
                self.max_stock = max_stock;
                updated_fields.push(String::from("max_stock"));
            },
            None => {}
        };
        match updated_product.stock_level {
            Some(stock_level) => {
                self.stock_level = stock_level;
                updated_fields.push(String::from("stock_level"));
            },
            None => {}
        };
        if updated_fields.len() == 0 {
            return Err("No fields to update");
        }
        self.version += 1;
        Ok(updated_fields)
    }
}

fn read_file() -> Vec<Product> {
    let mut product_list = Vec::<Product>::new();
    let mut reader = match ReaderBuilder::new().has_headers(true).from_path("data/products.csv") {
        Ok(x) => x,
        Err(_x) => panic!("Cannot read the input file")
    };
    for result in reader.deserialize::<NewProduct>() {
        let record = match result {
            Ok(x) => x,
            Err(x) => panic!("{:?}", x)
        };
        product_list.push(Product::from_new_product(record));
    }
    println!("Finished reading the file");
    product_list
}

#[derive(Deserialize, Debug)]
struct TableQuery {
    page_size: Option<usize>,
    page_index: Option<usize>
}

#[derive(Serialize)]
pub struct ProgramState {
    pub products: Vec<Product>
}

impl ProgramState {
    pub fn new() -> ProgramState {
        ProgramState {
            products: Vec::new()
        }
    }
}

#[derive(Serialize)]
struct CustomResponse {
    reason: String,
}

impl CustomResponse {
    pub fn new(message: String) -> CustomResponse {
        Self {
            reason: message
        }
    }
}

#[derive(Deserialize, Debug)]
struct IdQuery {
    id: String
}

#[derive(Deserialize, Debug)]
struct SearchQuery {
    search: String
}

#[post("/api/v1/products/{id}")]
async fn update_product(global_storage: web::Data<Arc<Mutex<ProgramState>>>, updated_product: web::Json<UpdatedProduct>, path_params: web::Path<IdQuery>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let products = &mut program_state.products;
    for product in products.into_iter() {
        if product.id != path_params.id {
            continue;
        }
        let updated_fields = match product.update_product(updated_product.0) {
            Ok(x) => (*x).to_vec(),
            Err(x) => return HttpResponse::Ok().json(CustomResponse::new(String::from(x)))
        };
        println!("{:?}", updated_fields);
        return HttpResponse::Ok().json(product);
    };
    HttpResponse::Ok().json(CustomResponse::new(String::from("Product not found")))
}

#[put("/api/v1/products/{id}/increment")]
async fn increment_product(global_storage: web::Data<Arc<Mutex<ProgramState>>>, path_params: web::Path<IdQuery>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let products = &mut program_state.products;
    for product in products.into_iter() {
        if product.id != path_params.id {
            continue;
        }
        product.stock_level += 1;
        product.version += 1;
        return HttpResponse::Ok().json(product);
    };
    HttpResponse::Ok().json(CustomResponse::new(String::from("Product not found")))
}

#[derive(Serialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct LevenshteinDistance {
    id: String,
    name: String,
    distance: usize
}

impl LevenshteinDistance {
    pub fn new(id: String, name: String, distance:usize) -> LevenshteinDistance {
        LevenshteinDistance {
            id: id,
            name: name,
            distance: distance
        }
    }
    pub fn calculate(str1: &String, str2: &String) -> usize {
        if str1 == str2 {
            return 0;
        }

        let n = str1.len();
        let m = str2.len();

        let mut column: Vec<usize> = (0..n + 1).collect();
        // TODO this probaly needs to use graphemes
        let a_vec: Vec<char> = str1.chars().collect();
        let b_vec: Vec<char> = str2.chars().collect();
        for i in 1..m + 1 {
            let previous = column;
            column = vec![0; n + 1];
            column[0] = i;
            for j in 1..n + 1 {
                let add = previous[j] + 1;
                let delete = column[j - 1] + 1;
                let mut change = previous[j - 1];
                if a_vec[j - 1] != b_vec[i - 1] {
                    change += 2;
                }
                else if change > 0 {
                    change -= 1;
                }
                column[j] = cmp::min(add, cmp::min(delete, change));
            }
        }
        column[n]
    }
}

#[get("/api/v1/products/search/{search}")]
async fn search_for_products(global_storage: web::Data<Arc<Mutex<ProgramState>>>, path_params: web::Path<SearchQuery>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let products = &mut program_state.products;
    let mut results: Vec<LevenshteinDistance> = Vec::new();
    let search_term = &path_params.search.to_lowercase();
    for product in products.into_iter() {
        let product_name = &product.name.to_lowercase();
        if search_term == product_name {
            results.push(LevenshteinDistance::new(product.id.clone(), product.name.clone(), 0));
            continue;
        }
        if product_name.starts_with(search_term) {
            results.push(LevenshteinDistance::new(product.id.clone(), product.name.clone(), 0));
            continue;
        }
        let distance = LevenshteinDistance::calculate(search_term, product_name);
        if distance <= 10 {
            results.push(LevenshteinDistance::new(product.id.clone(), product.name.clone(), distance));
        }
    }
    results.sort_by(|a, b| a.distance.cmp(&b.distance));
    HttpResponse::Ok().json(results.into_iter().take(10).collect::<Vec<LevenshteinDistance>>())
}

#[put("/api/v1/products/{id}/decrement")]
async fn decrement_product(global_storage: web::Data<Arc<Mutex<ProgramState>>>, path_params: web::Path<IdQuery>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let products = &mut program_state.products;
    for product in products.into_iter() {
        if product.id != path_params.id {
            continue;
        }
        product.stock_level -= 1;
        product.version += 1;
        return HttpResponse::Ok().json(product);
    };
    HttpResponse::Ok().json(CustomResponse::new(String::from("Product not found")))
}

#[get("/api/v1/products/{id}")]
async fn find_product_by_id(global_storage: web::Data<Arc<Mutex<ProgramState>>>, path_params: web::Path<IdQuery>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let products = &mut program_state.products;
    for product in products.into_iter() {
        if product.id != path_params.id {
            continue;
        }
        return HttpResponse::Ok().json(product);
    };
    HttpResponse::Ok().json(CustomResponse::new(String::from("Product not found")))
}

#[delete("/api/v1/products/{id}")]
async fn delete_product_by_id(global_storage: web::Data<Arc<Mutex<ProgramState>>>, path_params: web::Path<IdQuery>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let products = &mut program_state.products;
    for product in products.into_iter() {
        if product.id != path_params.id {
            continue;
        }
        product.delete();
        return HttpResponse::Ok().json(product);
    };
    HttpResponse::Ok().json(CustomResponse::new(String::from("Product not found")))
}

#[get("/api/v1/products/add")]
async fn add_product(global_storage: web::Data<Arc<Mutex<ProgramState>>>, new_product: web::Json<NewProduct>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    let product = Product::from_new_product(new_product.0);
    program_state.products.push(product.clone());
    HttpResponse::Ok().json(product)
}

#[get("/api/v1/products")]
async fn list_products(global_storage: web::Data<Arc<Mutex<ProgramState>>>, table_query: web::Query<TableQuery>) -> impl Responder {
    let program_state = &*global_storage.lock().unwrap();
    let page_size = match table_query.0.page_size {
        Some(x) => x,
        None => 10
    };
    let page_index = match table_query.0.page_index {
        Some(x) => x,
        None => 0
    };
    let products = &program_state.products;
    HttpResponse::Ok().json(products.into_iter().skip(page_index*page_size).take(page_size).collect::<Vec<&Product>>())
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    //add some global storage here
    let mut product_list = ProgramState::new();
    let mut current_product_list = read_file();
    product_list.products.append(&mut current_product_list);
    let global_storage = Arc::new(Mutex::new(product_list));
    HttpServer::new(move || 
        App::new()
            .data(global_storage.clone()) // add shared state
            .wrap(middleware::Compress::default()) // compresses the output
            .service(list_products) 
            .service(add_product)
            .service(update_product)
            .service(find_product_by_id)
            .service(increment_product)
            .service(decrement_product)
            .service(search_for_products)
            .service(delete_product_by_id)
            .service(web::resource("/ws").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "./static/")
            .index_file("index.html"))
        )
        .bind("127.0.0.1:8080")?
        .run()
        .await
}




/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(r: HttpRequest, stream: web::Payload, global_storage: web::Data<Arc<Mutex<ProgramState>>>) -> Result<HttpResponse, Error> {
    //println!("{:?}", r);
    let res = ws::start(MyWebSocket::new(global_storage), &r, stream);
    //println!("{:?}", res);
    res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    global_storage: web::Data<Arc<Mutex<ProgramState>>>
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        // process websocket messages
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
                let program_state = &*self.global_storage.lock().unwrap();
                ctx.text(String::from(serde_json::to_string(program_state)
                .unwrap()))
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWebSocket {
    fn new(global_storage: web::Data<Arc<Mutex<ProgramState>>>) -> Self {
        Self { 
            hb: Instant::now(),
            global_storage: global_storage
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}