use serde::Serialize;
use spring_batch_rs::core::item::ItemWriter;
use spring_batch_rs::item::rdbc::{DatabaseItemBinder, MySqlItemWriter};
use sqlx::{query_builder::Separated, MySql, MySqlPool};

/// Example data structure representing a product
#[derive(Clone, Serialize, Debug)]
struct Product {
    id: i32,
    name: String,
    category: String,
    price: f64,
    in_stock: bool,
}

/// Custom binder for Product items to MySQL database
struct ProductBinder;

impl DatabaseItemBinder<Product, MySql> for ProductBinder {
    /// Binds Product fields to MySQL query parameters
    fn bind(&self, item: &Product, mut query_builder: Separated<MySql, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.clone());
        query_builder.push_bind(item.category.clone());
        query_builder.push_bind(item.price);
        query_builder.push_bind(item.in_stock);
    }
}

/// Creates sample product data for demonstration
fn create_sample_products() -> Vec<Product> {
    vec![
        Product {
            id: 1,
            name: "Gaming Laptop".to_string(),
            category: "Electronics".to_string(),
            price: 1299.99,
            in_stock: true,
        },
        Product {
            id: 2,
            name: "Wireless Mouse".to_string(),
            category: "Electronics".to_string(),
            price: 29.99,
            in_stock: true,
        },
        Product {
            id: 3,
            name: "Mechanical Keyboard".to_string(),
            category: "Electronics".to_string(),
            price: 149.99,
            in_stock: false,
        },
        Product {
            id: 4,
            name: "4K Monitor".to_string(),
            category: "Electronics".to_string(),
            price: 399.99,
            in_stock: true,
        },
        Product {
            id: 5,
            name: "USB-C Hub".to_string(),
            category: "Accessories".to_string(),
            price: 59.99,
            in_stock: true,
        },
    ]
}

/// Demonstrates MySQL writer configuration and usage
fn demonstrate_mysql_writer() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐬 MySQL Item Writer Example");
    println!("============================");
    println!();

    // Create sample data
    let products = create_sample_products();
    println!("📦 Sample Products Created:");
    for product in &products {
        println!(
            "   • {} - ${:.2} ({})",
            product.name,
            product.price,
            if product.in_stock {
                "In Stock"
            } else {
                "Out of Stock"
            }
        );
    }
    println!();

    println!("🔧 MySQL Writer Configuration:");
    println!("   - Database: MySQL/MariaDB");
    println!("   - Connection: mysql://user:pass@localhost:3306/batch_db");
    println!("   - Table: products");
    println!("   - Columns: id, name, category, price, in_stock");
    println!("   - Batch Size: {} items per batch", products.len());
    println!();

    // This would be the actual implementation (commented out since we don't have a real DB):
    /*
    let pool = MySqlPool::connect("mysql://user:pass@localhost:3306/batch_db").await?;
    let binder = ProductBinder;

    // Create the MySQL writer using the builder pattern
    let writer = MySqlItemWriter::<Product>::new()
        .pool(&pool)
        .table("products")
        .add_column("id")
        .add_column("name")
        .add_column("category")
        .add_column("price")
        .add_column("in_stock")
        .item_binder(&binder);

    // Write the products to the database
    println!("💾 Writing products to MySQL database...");
    writer.write(&products)?;
    println!("✅ Successfully wrote {} products to MySQL!", products.len());
    */

    println!("💡 MySQL Writer Features:");
    println!("   • Efficient batch inserts with parameter binding");
    println!("   • Connection pooling for optimal performance");
    println!("   • Support for MySQL-specific data types");
    println!("   • Automatic transaction handling");
    println!("   • Comprehensive error handling and logging");
    println!("   • Generic DatabaseItemBinder for type safety");
    println!();

    println!("🔍 Usage Pattern:");
    println!("   1. Define your data structure with Serialize trait");
    println!("   2. Implement DatabaseItemBinder<YourType, MySql>");
    println!("   3. Create MySqlItemWriter with builder pattern");
    println!("   4. Configure pool, table, columns, and binder");
    println!("   5. Call write() with your data slice");
    println!();

    println!("📋 SQL Generated (example):");
    println!("   INSERT INTO products (id, name, category, price, in_stock)");
    println!("   VALUES (?, ?, ?, ?, ?), (?, ?, ?, ?, ?), ...");
    println!();

    Ok(())
}

/// Demonstrates advanced MySQL writer features
fn demonstrate_advanced_features() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced MySQL Writer Features");
    println!("=================================");
    println!();

    println!("🔧 Performance Optimizations:");
    println!("   • Batch parameter limit: 65,535 parameters");
    println!("   • Automatic chunking for large datasets");
    println!("   • Connection pooling with configurable limits");
    println!("   • Prepared statement reuse");
    println!();

    println!("🛡️  Error Handling:");
    println!("   • Constraint violation detection");
    println!("   • Connection failure recovery");
    println!("   • Detailed error logging with context");
    println!("   • Graceful degradation on partial failures");
    println!();

    println!("🎯 MySQL-Specific Features:");
    println!("   • Support for AUTO_INCREMENT columns");
    println!("   • DATETIME and TIMESTAMP handling");
    println!("   • JSON column support");
    println!("   • Binary data (BLOB) support");
    println!("   • Character set and collation handling");
    println!();

    println!("📊 Monitoring & Observability:");
    println!("   • Structured logging with log crate");
    println!("   • Execution time tracking");
    println!("   • Row count verification");
    println!("   • Connection pool metrics");
    println!();

    Ok(())
}

/// Demonstrates error scenarios and handling
fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Error Handling Examples");
    println!("===========================");
    println!();

    println!("🔴 Common Error Scenarios:");
    println!("   • Connection timeout or failure");
    println!("   • Primary key constraint violation");
    println!("   • Foreign key constraint violation");
    println!("   • Data type mismatch");
    println!("   • Table or column not found");
    println!("   • Insufficient permissions");
    println!();

    println!("🛠️  Error Recovery Strategies:");
    println!("   • Retry with exponential backoff");
    println!("   • Skip invalid records with logging");
    println!("   • Batch size reduction on parameter limits");
    println!("   • Connection pool health checks");
    println!();

    println!("📝 Error Logging Format:");
    println!("   ERROR Failed to write items to MySQL table products: ...");
    println!("   DEBUG Successfully wrote 1000 items to MySQL table products");
    println!();

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🌟 Spring Batch RS - MySQL Writer Example");
    println!("==========================================");
    println!();

    // Run demonstrations
    demonstrate_mysql_writer()?;
    demonstrate_advanced_features()?;
    demonstrate_error_handling()?;

    println!("🎉 MySQL Writer Example Complete!");
    println!();
    println!("💡 Next Steps:");
    println!("   • Set up a MySQL database instance");
    println!("   • Update connection string in your code");
    println!("   • Create the target table schema");
    println!("   • Implement your custom DatabaseItemBinder");
    println!("   • Integrate with your batch processing pipeline");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sample_products() {
        let products = create_sample_products();
        assert_eq!(products.len(), 5);
        assert_eq!(products[0].name, "Gaming Laptop");
        assert_eq!(products[0].price, 1299.99);
        assert!(products[0].in_stock);
    }

    #[test]
    fn test_product_binder_interface() {
        // Test that our binder implements the required trait
        let _binder: Box<dyn DatabaseItemBinder<Product, MySql>> = Box::new(ProductBinder);
    }

    #[test]
    fn test_product_serialization() {
        let product = Product {
            id: 1,
            name: "Test Product".to_string(),
            category: "Test".to_string(),
            price: 99.99,
            in_stock: true,
        };

        // Test that product can be serialized (required for ItemWriter)
        let _json = serde_json::to_string(&product).unwrap();
    }
}
