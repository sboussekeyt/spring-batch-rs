use anyhow::Result;
use sea_orm::{
    entity::prelude::*,
    ActiveValue::{NotSet, Set},
    Database, DatabaseConnection,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{
    core::{
        item::{ItemProcessor, ItemProcessorResult, ItemReader},
        job::{Job, JobBuilder},
        step::StepBuilder,
    },
    item::orm::OrmItemWriterBuilder,
    BatchError,
};
use std::cell::Cell;

/// Example entity representing a Product in the database
#[derive(Debug, Clone, DeriveEntityModel, Deserialize, Serialize, PartialEq)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub category: String,
    pub price: Decimal,
    pub in_stock: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Business domain object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDto {
    pub name: String,
    pub category: String,
    pub price: f64,
    pub in_stock: bool,
}

/// Processor for converting business DTOs to SeaORM active models
#[derive(Default)]
pub struct ProductDtoToActiveModelProcessor;

impl ItemProcessor<ProductDto, ActiveModel> for ProductDtoToActiveModelProcessor {
    fn process(&self, item: &ProductDto) -> ItemProcessorResult<ActiveModel> {
        let active_model = ActiveModel {
            id: NotSet, // Auto-generated
            name: Set(item.name.clone()),
            category: Set(item.category.clone()),
            price: Set(Decimal::from_f64_retain(item.price).unwrap()),
            in_stock: Set(item.in_stock),
            created_at: Set(DateTimeUtc::default()),
        };
        Ok(active_model)
    }
}

/// Simple reader for providing test data
struct ProductDtoReader {
    products: Vec<ProductDto>,
    position: Cell<usize>,
}

impl ItemReader<ProductDto> for ProductDtoReader {
    fn read(&self) -> Result<Option<ProductDto>, BatchError> {
        let pos = self.position.get();
        if pos < self.products.len() {
            let product = self.products[pos].clone();
            self.position.set(pos + 1);
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }
}

/// Reader for providing SeaORM active models directly
struct ActiveModelReader {
    active_models: Vec<ActiveModel>,
    position: Cell<usize>,
}

impl ItemReader<ActiveModel> for ActiveModelReader {
    fn read(&self) -> Result<Option<ActiveModel>, BatchError> {
        let pos = self.position.get();
        if pos < self.active_models.len() {
            let model = self.active_models[pos].clone();
            self.position.set(pos + 1);
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}

/// Pass-through processor
#[derive(Default)]
struct PassThroughProcessor<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone> ItemProcessor<T, T> for PassThroughProcessor<T> {
    fn process(&self, item: &T) -> ItemProcessorResult<T> {
        Ok(item.clone())
    }
}

/// Set up test database
async fn setup_database() -> Result<DatabaseConnection> {
    let db = Database::connect("sqlite::memory:").await?;

    // Create table
    let create_table_sql = r#"
        CREATE TABLE products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            price DECIMAL(10,2) NOT NULL,
            in_stock BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#;

    db.execute_unprepared(create_table_sql).await?;
    Ok(db)
}

/// Example 1: Using business DTOs with processor transformation
async fn example_with_business_dtos(db: &DatabaseConnection) -> Result<()> {
    println!("=== Example 1: Business DTOs with Processor Transformation ===");

    // Create test data as business DTOs
    let test_data = vec![
        ProductDto {
            name: "Gaming Laptop".to_string(),
            category: "Electronics".to_string(),
            price: 1299.99,
            in_stock: true,
        },
        ProductDto {
            name: "Office Chair".to_string(),
            category: "Furniture".to_string(),
            price: 299.99,
            in_stock: false,
        },
        ProductDto {
            name: "Coffee Mug".to_string(),
            category: "Kitchen".to_string(),
            price: 12.99,
            in_stock: true,
        },
    ];

    // Create reader
    let reader = ProductDtoReader {
        products: test_data,
        position: Cell::new(0),
    };

    // Create writer - no mapper needed!
    let writer = OrmItemWriterBuilder::<ActiveModel>::new()
        .connection(db)
        .build();

    // Use processor to convert DTOs to active models
    let processor = ProductDtoToActiveModelProcessor::default();

    // Run job
    let step = StepBuilder::new("write_business_dtos")
        .chunk::<ProductDto, ActiveModel>(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    match result {
        Ok(_) => {
            let step_execution = job.get_step_execution("write_business_dtos").unwrap();
            println!(
                "✓ Successfully wrote {} items using business DTOs with processor",
                step_execution.write_count
            );
        }
        Err(e) => println!("✗ Job failed: {:?}", e),
    }

    Ok(())
}

/// Example 2: Working directly with SeaORM active models
async fn example_with_direct_entities(db: &DatabaseConnection) -> Result<()> {
    println!("\n=== Example 2: Direct SeaORM Active Models ===");

    // Create test data as SeaORM active models directly
    let test_data = vec![
        ActiveModel {
            id: NotSet,
            name: Set("Wireless Headphones".to_string()),
            category: Set("Electronics".to_string()),
            price: Set(Decimal::from_f64_retain(199.99).unwrap()),
            in_stock: Set(true),
            created_at: Set(DateTimeUtc::default()),
        },
        ActiveModel {
            id: NotSet,
            name: Set("Standing Desk".to_string()),
            category: Set("Furniture".to_string()),
            price: Set(Decimal::from_f64_retain(599.99).unwrap()),
            in_stock: Set(true),
            created_at: Set(DateTimeUtc::default()),
        },
    ];

    // Create reader
    let reader = ActiveModelReader {
        active_models: test_data,
        position: Cell::new(0),
    };

    // Create writer - works directly with active models!
    let writer = OrmItemWriterBuilder::<ActiveModel>::new()
        .connection(db)
        .build();

    let processor = PassThroughProcessor {
        _phantom: std::marker::PhantomData,
    };

    // Run job
    let step = StepBuilder::new("write_direct_entities")
        .chunk::<ActiveModel, ActiveModel>(2)
        .reader(&reader)
        .processor(&processor)
        .writer(&writer)
        .build();

    let job = JobBuilder::new().start(&step).build();
    let result = job.run();

    match result {
        Ok(_) => {
            let step_execution = job.get_step_execution("write_direct_entities").unwrap();
            println!(
                "✓ Successfully wrote {} items using direct active models",
                step_execution.write_count
            );
        }
        Err(e) => println!("✗ Job failed: {:?}", e),
    }

    Ok(())
}

/// Verify the data was written correctly
async fn verify_data(db: &DatabaseConnection) -> Result<()> {
    println!("\n=== Verifying Written Data ===");

    let products: Vec<Model> = Entity::find().all(db).await?;
    println!("Total products in database: {}", products.len());

    for product in products {
        println!(
            "- {} ({}) - ${:.2} - {}",
            product.name,
            product.category,
            product.price,
            if product.in_stock {
                "In Stock"
            } else {
                "Out of Stock"
            }
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    env_logger::init();

    println!("ORM Writer Example - Simplified Direct Approach");
    println!("===============================================");

    // Set up database
    let db = setup_database().await?;

    // Example 1: Business DTOs with processor transformation
    example_with_business_dtos(&db).await?;

    // Example 2: Direct active model usage
    example_with_direct_entities(&db).await?;

    // Verify the results
    verify_data(&db).await?;

    println!("\n=== Summary ===");
    println!("✓ ORM Writer now uses a simplified direct approach:");
    println!("  1. Business DTOs → Active Models: Use processors for transformation");
    println!("     - Convert DTOs to active models in the processor");
    println!("     - Clean separation of concerns");
    println!("     - Flexible data transformation");
    println!("  2. Direct Active Models: Work directly with ORM entities");
    println!("     - No transformation needed");
    println!("     - Simple and efficient");
    println!("     - Best for CRUD operations");
    println!("  Both approaches use the same OrmItemWriter - no mappers needed!");

    Ok(())
}
