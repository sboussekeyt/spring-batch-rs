use sea_orm::{
    entity::prelude::*, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use spring_batch_rs::{core::item::ItemReader, item::orm::OrmItemReaderBuilder};

/// Example entity representing a User in the database
#[derive(Debug, Clone, DeriveEntityModel, Deserialize, Serialize, PartialEq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// DTO for processed user data (used for demonstration of transformation)
#[derive(Debug, Clone)]
pub struct UserDto {
    pub id: i32,
    pub display_name: String,
    pub contact_email: String,
    pub is_active: bool,
}

/// Helper function to transform a Model to UserDto
fn transform_to_dto(model: Model) -> UserDto {
    UserDto {
        id: model.id,
        display_name: format!("User: {}", model.name),
        contact_email: model.email,
        is_active: model.active,
    }
}

/// Sets up a test database with sample data
async fn setup_test_database() -> Result<DatabaseConnection, DbErr> {
    // Connect to an in-memory SQLite database
    let db = Database::connect("sqlite::memory:").await?;

    // Create the users table
    let create_table_sql = r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            active BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    "#;

    db.execute_unprepared(create_table_sql).await?;

    // Insert sample data
    let insert_data_sql = r#"
        INSERT INTO users (name, email, active, created_at) VALUES
        ('Alice Johnson', 'alice@example.com', 1, '2024-01-01 10:00:00'),
        ('Bob Smith', 'bob@example.com', 1, '2024-01-02 11:00:00'),
        ('Charlie Brown', 'charlie@example.com', 0, '2024-01-03 12:00:00'),
        ('Diana Prince', 'diana@example.com', 1, '2024-01-04 13:00:00'),
        ('Eve Wilson', 'eve@example.com', 1, '2024-01-05 14:00:00'),
        ('Frank Miller', 'frank@example.com', 0, '2024-01-06 15:00:00'),
        ('Grace Lee', 'grace@example.com', 1, '2024-01-07 16:00:00'),
        ('Henry Davis', 'henry@example.com', 1, '2024-01-08 17:00:00'),
        ('Ivy Chen', 'ivy@example.com', 1, '2024-01-09 18:00:00'),
        ('Jack Taylor', 'jack@example.com', 0, '2024-01-10 19:00:00')
    "#;

    db.execute_unprepared(insert_data_sql).await?;

    Ok(db)
}

/// Example 1: Reading all users without pagination
async fn example_read_all_users(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Reading all users ===");

    // Create a query to select all users
    let query = Entity::find();

    // Create the reader without pagination
    let reader = OrmItemReaderBuilder::new()
        .connection(db)
        .query(query)
        .build();

    // Read all users
    let mut count = 0;
    while let Some(user) = reader.read()? {
        println!("User {}: {} ({})", user.id, user.name, user.email);
        count += 1;
    }

    println!("Total users read: {}\n", count);
    Ok(())
}

/// Example 2: Reading active users with pagination
async fn example_read_active_users_paginated(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 2: Reading active users with pagination ===");

    // Create a query to select only active users, ordered by ID
    let query = Entity::find()
        .filter(Column::Active.eq(true))
        .order_by_asc(Column::Id);

    // Create the reader with pagination (3 users per page)
    let reader = OrmItemReaderBuilder::new()
        .connection(db)
        .query(query)
        .page_size(3)
        .build();

    // Read active users
    let mut count = 0;
    while let Some(user) = reader.read()? {
        println!(
            "Active User {}: {} ({}) - Created: {}",
            user.id, user.name, user.email, user.created_at
        );
        count += 1;
    }

    println!("Total active users read: {}\n", count);
    Ok(())
}

/// Example 3: Reading users with transformation using helper function
async fn example_read_users_with_transformation(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 3: Reading users with manual transformation ===");

    // Create a query to select users ordered by name
    let query = Entity::find().order_by_asc(Column::Name);

    // Create the reader with pagination
    let reader = OrmItemReaderBuilder::new()
        .connection(db)
        .query(query)
        .page_size(4)
        .build();

    // Read and transform users manually
    let mut count = 0;
    while let Some(user) = reader.read()? {
        let user_dto = transform_to_dto(user);
        println!(
            "DTO {}: {} ({}) - Active: {}",
            user_dto.id, user_dto.display_name, user_dto.contact_email, user_dto.is_active
        );
        count += 1;
    }

    println!("Total users transformed: {}\n", count);
    Ok(())
}

/// Example 4: Reading users with complex filtering
async fn example_read_users_with_complex_filter(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 4: Reading users with complex filtering ===");

    // Create a complex query: active users whose names start with specific letters
    let query = Entity::find()
        .filter(Column::Active.eq(true))
        .filter(
            Column::Name
                .like("A%")
                .or(Column::Name.like("D%"))
                .or(Column::Name.like("G%")),
        )
        .order_by_asc(Column::Name);

    // Create the reader
    let reader = OrmItemReaderBuilder::new()
        .connection(db)
        .query(query)
        .build();

    // Read filtered users
    let mut count = 0;
    while let Some(user) = reader.read()? {
        println!("Filtered User {}: {} ({})", user.id, user.name, user.email);
        count += 1;
    }

    println!("Total filtered users read: {}\n", count);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("ORM Item Reader Examples");
    println!("========================\n");

    // Setup test database
    let db = setup_test_database().await?;

    // Run examples
    example_read_all_users(&db).await?;
    example_read_active_users_paginated(&db).await?;
    example_read_users_with_transformation(&db).await?;
    example_read_users_with_complex_filter(&db).await?;

    println!("All examples completed successfully!");

    Ok(())
}
