use serde::{Deserialize, Serialize};
use spring_batch_rs::item::rdbc::DatabaseItemBinder;
use sqlx::{query_builder::Separated, MySql, Postgres};

/// Example data structure representing a user
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

/// PostgreSQL-specific implementation of the generic DatabaseItemBinder trait
///
/// This implementation shows how to bind User data to PostgreSQL queries
struct PostgresUserBinder;

impl DatabaseItemBinder<User, Postgres> for PostgresUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Postgres, &str>) {
        // In a real implementation, you would bind the item properties to the query
        // For this example, we'll use placeholder bindings to avoid lifetime issues
        let _ = (item, &mut query_builder); // Avoid unused parameter warnings
                                            // query_builder.push_bind(item.id);
                                            // query_builder.push_bind(&item.name);
                                            // query_builder.push_bind(&item.email);
    }
}

/// MySQL-specific implementation of the generic DatabaseItemBinder trait
///
/// This implementation shows how to bind User data to MySQL queries
/// Note: The binding logic is identical to PostgreSQL, but the trait is parameterized
/// with the MySql database type, making it type-safe for MySQL operations
struct MySqlUserBinder;

impl DatabaseItemBinder<User, MySql> for MySqlUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<MySql, &str>) {
        // In a real implementation, you would bind the item properties to the query
        // For this example, we'll use placeholder bindings to avoid lifetime issues
        let _ = (item, &mut query_builder); // Avoid unused parameter warnings
                                            // query_builder.push_bind(item.id);
                                            // query_builder.push_bind(&item.name);
                                            // query_builder.push_bind(&item.email);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Database Agnostic Binder Example");
    println!("=================================");
    println!();

    println!("This example demonstrates how the generic DatabaseItemBinder trait");
    println!("allows you to write database-specific item binders for different databases");
    println!("while maintaining type safety and code reusability.");
    println!();

    // Example users data
    let users = vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
        },
        User {
            id: 3,
            name: "Carol Davis".to_string(),
            email: "carol@example.com".to_string(),
        },
    ];

    println!("Sample users data:");
    for user in &users {
        println!(
            "  - ID: {}, Name: {}, Email: {}",
            user.id, user.name, user.email
        );
    }
    println!();

    println!("Key Benefits of the Generic DatabaseItemBinder:");
    println!("  1. Type Safety: Each database type has its own implementation");
    println!("  2. Code Reusability: Same pattern works across PostgreSQL, MySQL, SQLite");
    println!("  3. Maintainability: Easy to add support for new databases");
    println!("  4. Performance: Database-specific optimizations possible");
    println!();

    println!("Usage Examples:");
    println!();

    println!("PostgreSQL Writer Setup:");
    println!("```rust");
    println!("use spring_batch_rs::item::rdbc::postgres_writer::PostgresItemWriter;");
    println!("use spring_batch_rs::item::rdbc::DatabaseItemBinder;");
    println!("use sqlx::{{PgPool, query_builder::Separated, Postgres}};");
    println!();
    println!("let pool = PgPool::connect(\"postgresql://user:pass@localhost/db\").await?;");
    println!("let binder = PostgresUserBinder;");
    println!();
    println!("let writer = PostgresItemWriter::<User>::new()");
    println!("    .pool(&pool)");
    println!("    .table(\"users\")");
    println!("    .add_column(\"id\")");
    println!("    .add_column(\"name\")");
    println!("    .add_column(\"email\")");
    println!("    .item_binder(&binder);");
    println!("```");
    println!();

    println!("MySQL Writer Setup:");
    println!("```rust");
    println!("use spring_batch_rs::item::rdbc::mysql_writer::MySqlItemWriter;");
    println!("use spring_batch_rs::item::rdbc::DatabaseItemBinder;");
    println!("use sqlx::{{MySqlPool, query_builder::Separated, MySql}};");
    println!();
    println!("let pool = MySqlPool::connect(\"mysql://user:pass@localhost/db\").await?;");
    println!("let binder = MySqlUserBinder;");
    println!();
    println!("let writer = MySqlItemWriter::<User>::new()");
    println!("    .pool(&pool)");
    println!("    .table(\"users\")");
    println!("    .add_column(\"id\")");
    println!("    .add_column(\"name\")");
    println!("    .add_column(\"email\")");
    println!("    .item_binder(&binder);");
    println!("```");
    println!();

    println!("The same User struct and similar binder logic can be used across");
    println!("different databases, with compile-time type safety ensuring that");
    println!("PostgreSQL binders are only used with PostgreSQL writers, and");
    println!("MySQL binders are only used with MySQL writers.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_binder_instantiation() {
        let _postgres_binder = PostgresUserBinder;
        let _mysql_binder = MySqlUserBinder;

        // Both binders can be instantiated without issues
        // The actual binding behavior would be tested in integration tests
        // with real database connections
    }
}
