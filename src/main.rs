extern crate csv;
extern crate actix_web;
extern crate serde;
extern crate serde_json;

#[macro_use]
use serde::{Serialize};

use csv::Reader;
use serde::{Deserialize};
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use std::sync::{Arc, Mutex, MutexGuard};
use std::io;
use actix_files as fs;


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
            .service(fs::Files::new("/", "./static/")
            .index_file("index.html"))
        )
        .bind("127.0.0.1:8080")?
        .run()
        .await
}