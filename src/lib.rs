extern crate serde;

pub mod main;

#[cfg(test)]
pub mod tests {
    use super::*;
    use main::{Product, NewProduct};

    #[test]
    pub fn product_stock_count() {
        let product: Product = Product::new(String::from("Cheese"), 555, String::from("1111111111111"), String::new(), String::new(), 10, 20);
        assert_eq!(product.stock_level, 10);
    }

    #[test]
    pub fn product_set_name() {
        let product: Product = Product::new(String::from("Cheese"), 555, String::from("1111111111111"), String::new(), String::new(), 10, 20);
        assert_eq!(product.name, String::from("Cheese"));
    }

    #[test]
    pub fn product_from_new_product() {
        let product: Product = Product::from_new_product(NewProduct {
            name: String::from("Cheese"),
            price: 5.55,
            barcode: String::from("11111111111111"),
            department: String::new(),
            supplier: String::new(),
            stock_level: 5,
            max_stock: 10
        });
        assert_eq!(product.name, String::from("Cheese"));
        assert!(product.id != String::new());
    }
}