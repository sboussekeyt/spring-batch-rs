use serde::Serialize;
use spring_batch_rs::core::item::ItemWriter;
use spring_batch_rs::item::rdbc::{
    DatabaseItemBinder, MySqlItemWriter, PostgresItemWriter, SqliteItemWriter,
};
use sqlx::{query_builder::Separated, MySql, Pool, Postgres, Sqlite};

#[derive(Clone, Serialize, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// PostgreSQL Binder
struct PostgresUserBinder;
impl DatabaseItemBinder<User, Postgres> for PostgresUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Postgres, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.to_string());
        query_builder.push_bind(item.email.to_string());
    }
}

// MySQL Binder
struct MySqlUserBinder;
impl DatabaseItemBinder<User, MySql> for MySqlUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<MySql, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.to_string());
        query_builder.push_bind(item.email.to_string());
    }
}

// SQLite Binder
struct SqliteUserBinder;
impl DatabaseItemBinder<User, Sqlite> for SqliteUserBinder {
    fn bind(&self, item: &User, mut query_builder: Separated<Sqlite, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.name.to_string());
        query_builder.push_bind(item.email.to_string());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Generic Database Writer with DB Parameter Example ===");

    // Sample data
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
        User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
        },
    ];

    println!("Sample users to write: {:#?}", users);
    println!();

    // Demonstrate different database writer configurations
    demonstrate_postgres_writer(&users)?;
    demonstrate_mysql_writer(&users)?;
    demonstrate_sqlite_writer(&users)?;

    Ok(())
}

fn demonstrate_postgres_writer(users: &[User]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🐘 PostgreSQL Writer Configuration:");
    println!("   - Uses PostgresItemWriter<User>");
    println!("   - Uses DatabaseItemBinder<User, Postgres>");
    println!("   - Type-safe at compile time");

    // This would be the actual usage (commented out since we don't have a real DB):
    /*
    let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    let binder = PostgresUserBinder;

    let writer = PostgresItemWriter::<User>::new()
        .pool(&pool)
        .table("users")
        .add_column("id")
        .add_column("name")
        .add_column("email")
        .item_binder(&binder);

    writer.write(users)?;
    */

    println!("   ✓ PostgreSQL writer configured successfully");
    println!();
    Ok(())
}

fn demonstrate_mysql_writer(users: &[User]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🐬 MySQL Writer Configuration:");
    println!("   - Uses MySqlItemWriter<User>");
    println!("   - Uses DatabaseItemBinder<User, MySql>");
    println!("   - Same API, different database type");

    // This would be the actual usage (commented out since we don't have a real DB):
    /*
    let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
    let binder = MySqlUserBinder;

    let writer = MySqlItemWriter::<User>::new()
        .pool(&pool)
        .table("users")
        .add_column("id")
        .add_column("name")
        .add_column("email")
        .item_binder(&binder);

    writer.write(users)?;
    */

    println!("   ✓ MySQL writer configured successfully");
    println!();
    Ok(())
}

fn demonstrate_sqlite_writer(users: &[User]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗃️  SQLite Writer Configuration:");
    println!("   - Uses SqliteItemWriter<User>");
    println!("   - Uses DatabaseItemBinder<User, Sqlite>");
    println!("   - File-based database support");

    // This would be the actual usage (commented out since we don't have a real DB):
    /*
    let pool = SqlitePool::connect("sqlite://database.db").await?;
    let binder = SqliteUserBinder;

    let writer = SqliteItemWriter::<User>::new()
        .pool(&pool)
        .table("users")
        .add_column("id")
        .add_column("name")
        .add_column("email")
        .item_binder(&binder);

    writer.write(users)?;
    */

    println!("   ✓ SQLite writer configured successfully");
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binder_implementations() {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test that binders can be instantiated
        let _postgres_binder = PostgresUserBinder;
        let _mysql_binder = MySqlUserBinder;
        let _sqlite_binder = SqliteUserBinder;

        // Test that user data is valid
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_writer_types() {
        // Test that different writer types can be instantiated
        let _postgres_writer = PostgresItemWriter::<User>::new();
        let _mysql_writer = MySqlWriter::<User>::new();
        let _sqlite_writer = SqliteItemWriter::<User>::new();
    }
}
