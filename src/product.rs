pub struct Product {
    pub name: String,
    pub stock_level: i64,
}

impl Product {
    pub fn new(name: String, stock_level: i64) -> Product {
        Product {
            name: name,
            stock_level: stock_level 
        }
    }
}