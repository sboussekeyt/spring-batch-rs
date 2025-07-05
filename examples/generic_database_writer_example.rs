use serde::{Deserialize, Serialize};
use spring_batch_rs::item::rdbc::DatabaseItemBinder;
use sqlx::{query_builder::Separated, MySql, Postgres};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// PostgreSQL-specific binder using the generic DatabaseItemBinder trait
struct PostgresUserBinder;

impl DatabaseItemBinder<User, Postgres> for PostgresUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Postgres, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.to_string());
        query_builder.push_bind(item.email.to_string());
    }
}

// MySQL-specific binder using the same generic DatabaseItemBinder trait
struct MySqlUserBinder;

impl DatabaseItemBinder<User, MySql> for MySqlUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<MySql, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.to_string());
        query_builder.push_bind(item.email.to_string());
    }
}

/// This example demonstrates how the `DatabaseItemBinder` trait can be implemented
/// for different database types, providing a generic interface while maintaining
/// type safety.
///
/// The `PostgresItemWriter` uses `DatabaseItemBinder<User, Postgres>`, and
/// a hypothetical `MySqlItemWriter` would use `DatabaseItemBinder<User, MySql>`.
/// This ensures that you can't accidentally use a PostgreSQL binder with a MySQL writer.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example with PostgreSQL
    println!("Setting up PostgreSQL example...");

    // In a real application, you would connect to an actual database
    let _postgres_connection_string = "postgresql://user:password@localhost:5432/database";

    // Simulate PostgreSQL usage (commented out to avoid requiring actual DB connection)
    /*
    let pg_pool = PgPool::connect(postgres_connection_string).await?;
    let postgres_binder = PostgresUserBinder;

    let postgres_writer = PostgresItemWriter::new()
        .pool(&pg_pool)
        .table("users")
        .add_column("id")
        .add_column("name")
        .add_column("email")
        .item_binder(&postgres_binder);

    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    postgres_writer.write(&users)?;
    println!("Successfully wrote {} users to PostgreSQL", users.len());
    */

    // Demonstrate type safety - this would be a compile-time error:
    // let wrong_writer = PostgresItemWriter::new()
    //     .item_binder(&MySqlUserBinder); // ❌ Compile error: type mismatch!

    println!("✅ PostgreSQL binder implements DatabaseItemBinder<User, Postgres>");
    println!("✅ MySQL binder implements DatabaseItemBinder<User, MySql>");
    println!("✅ Type safety prevents mixing binders between different databases");
    println!("✅ Same trait interface works across all SQLx-supported databases");

    // Show the trait signatures for clarity
    println!("\nTrait implementations:");
    println!("- PostgresUserBinder: DatabaseItemBinder<User, Postgres>");
    println!("- MySqlUserBinder: DatabaseItemBinder<User, MySql>");
    println!("\nThis design provides:");
    println!("1. Code reusability - same pattern works for all databases");
    println!("2. Type safety - prevents mixing incompatible binders");
    println!("3. Database-specific optimizations - each impl can be optimized");
    println!("4. Maintainability - easy to add support for new databases");

    Ok(())
}
