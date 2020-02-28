#[macro_use]
extern crate serde;

pub mod main;

#[cfg(test)]
pub mod tests {
    use super::*;
    use main::Product;

    #[test]
    pub fn product_stock_count() {
        let product: Product = Product::new(String::from("Cheese"), 10, 20);
        assert_eq!(product.stock_level, 10);
    }

    #[test]
    pub fn product_set_name() {
        let product: Product = Product::new(String::from("Cheese"), 10, 20);
        assert_eq!(product.name, String::from("Cheese"));
    }
}