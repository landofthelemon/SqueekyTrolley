extern crate csv;

use csv::Reader;
use serde::{Deserialize};

fn main() {
    read_file();
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
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

fn read_file() {
    let mut reader = match Reader::from_path("data/products.csv") {
        Ok(x) => x,
        Err(x) => panic!("Cannot read the input file")
    };
    for result in reader.deserialize::<Product>() {
        let record = match result {
            Ok(x) => println!("{} {}/{}", x.name, x.stock_level, x.max_stock),
            Err(x) => panic!("{:?}", x)
        };
    }
    println!("Finished reading the file");
}
