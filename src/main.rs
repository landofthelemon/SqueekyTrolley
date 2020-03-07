extern crate csv;
extern crate actix_web;
extern crate serde;
extern crate serde_json;

#[macro_use]
use serde::{Serialize};

use std::time::{Duration, Instant};
use actix::prelude::*;

use csv::Reader;
use serde::{Deserialize};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::{Arc, Mutex};
use std::io;
use actix_files as fs;
use actix_web_actors::ws;


#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Product {
    pub name: String,
    #[serde(rename = "current_stock")]
    pub stock_level: i64,
    pub max_stock: i64
}

impl Product {
    pub fn new(name: String, stock_level: i64, max_stock: i64) -> Product {
        Product {
            name: name,
            stock_level: stock_level,
            max_stock: max_stock
        }
    }
}

fn read_file() -> Vec<Product> {
    let mut product_list = Vec::<Product>::new();
    let mut reader = match Reader::from_path("data/products.csv") {
        Ok(x) => x,
        Err(x) => panic!("Cannot read the input file")
    };
    for result in reader.deserialize::<Product>() {
        let record = match result {
            Ok(x) => x,
            Err(x) => panic!("{:?}", x)
        };
        product_list.push(record);
    }
    println!("Finished reading the file");
    product_list
}

#[derive(Serialize)]
pub struct ProgramState {
    pub list: Vec<Product>
}

impl ProgramState {
    pub fn new() -> ProgramState {
        ProgramState {
            list: Vec::new()
        }
    }
}
#[get("/api/v1/products/add")]
async fn add(global_storage: web::Data<Arc<Mutex<ProgramState>>>) -> impl Responder {
    let program_state = &mut global_storage.lock().unwrap();
    program_state.list.push(Product::new(String::from("Cheese"), 10, 20));
    let text = format!("{} products", program_state.list.len());
    HttpResponse::Ok().body(text)
}

#[get("/api/v1/products")]
async fn index(global_storage: web::Data<Arc<Mutex<ProgramState>>>) -> impl Responder {
    let program_state = &*global_storage.lock().unwrap();
    HttpResponse::Ok().json(program_state)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    //add some global storage here
    let mut product_list = ProgramState::new();
    let mut current_product_list = read_file();
    product_list.list.append(&mut current_product_list);
    let global_storage = Arc::new(Mutex::new(product_list));
    HttpServer::new(move || 
        App::new()
            .data(global_storage.clone()) // add shared state
            .service(index)
            .service(add)
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