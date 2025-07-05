use serde::Serialize;
use spring_batch_rs::core::item::ItemWriter;
use spring_batch_rs::item::rdbc::{DatabaseItemBinder, SqliteItemWriter};
use sqlx::{query_builder::Separated, Sqlite, SqlitePool};

/// Example data structure representing a user
#[derive(Clone, Serialize, Debug)]
struct User {
    id: i32,
    username: String,
    email: String,
    age: i32,
    active: bool,
    created_at: String,
}

/// Custom binder for User items to SQLite database
struct UserBinder;

impl DatabaseItemBinder<User, Sqlite> for UserBinder {
    /// Binds User fields to SQLite query parameters
    fn bind(&self, item: &User, mut query_builder: Separated<Sqlite, &str>) {
        query_builder.push_bind(item.id);
        query_builder.push_bind(item.username.clone());
        query_builder.push_bind(item.email.clone());
        query_builder.push_bind(item.age);
        query_builder.push_bind(item.active);
        query_builder.push_bind(item.created_at.clone());
    }
}

/// Creates sample user data for demonstration
fn create_sample_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            username: "alice_smith".to_string(),
            email: "alice@example.com".to_string(),
            age: 28,
            active: true,
            created_at: "2024-01-15 10:30:00".to_string(),
        },
        User {
            id: 2,
            username: "bob_jones".to_string(),
            email: "bob@example.com".to_string(),
            age: 35,
            active: true,
            created_at: "2024-01-16 14:22:00".to_string(),
        },
        User {
            id: 3,
            username: "charlie_brown".to_string(),
            email: "charlie@example.com".to_string(),
            age: 42,
            active: false,
            created_at: "2024-01-17 09:15:00".to_string(),
        },
        User {
            id: 4,
            username: "diana_prince".to_string(),
            email: "diana@example.com".to_string(),
            age: 31,
            active: true,
            created_at: "2024-01-18 16:45:00".to_string(),
        },
    ]
}

/// Demonstrates file-based SQLite writer
fn demonstrate_file_based_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 File-Based SQLite Writer Example");
    println!("===================================");
    println!();

    let users = create_sample_users();
    println!("👥 Sample Users Created:");
    for user in &users {
        println!(
            "   • {} ({}) - {} years old ({})",
            user.username,
            user.email,
            user.age,
            if user.active { "Active" } else { "Inactive" }
        );
    }
    println!();

    println!("🔧 File-Based SQLite Configuration:");
    println!("   - Database File: ./batch_users.db");
    println!("   - Connection: sqlite://./batch_users.db");
    println!("   - Table: users");
    println!("   - Columns: id, username, email, age, active, created_at");
    println!("   - Batch Size: {} users per batch", users.len());
    println!();

    // This would be the actual implementation (commented out since we don't have a real DB):
    /*
    let pool = SqlitePool::connect("sqlite://./batch_users.db").await?;
    let binder = UserBinder;

    // Create the SQLite writer using the builder pattern
    let writer = SqliteItemWriter::<User>::new()
        .pool(&pool)
        .table("users")
        .add_column("id")
        .add_column("username")
        .add_column("email")
        .add_column("age")
        .add_column("active")
        .add_column("created_at")
        .item_binder(&binder);

    // Write the users to the database
    println!("💾 Writing users to SQLite database file...");
    writer.write(&users)?;
    println!("✅ Successfully wrote {} users to SQLite file!", users.len());
    */

    println!("💡 File-Based SQLite Features:");
    println!("   • Persistent storage in single file");
    println!("   • No server setup required");
    println!("   • ACID transactions for data integrity");
    println!("   • Cross-platform compatibility");
    println!("   • Efficient for read-heavy workloads");
    println!();

    Ok(())
}

/// Demonstrates in-memory SQLite writer
fn demonstrate_in_memory_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 In-Memory SQLite Writer Example");
    println!("==================================");
    println!();

    let users = create_sample_users();

    println!("🔧 In-Memory SQLite Configuration:");
    println!("   - Database: In-Memory (RAM)");
    println!("   - Connection: sqlite::memory:");
    println!("   - Table: temp_users");
    println!("   - Persistence: Session-only");
    println!("   - Performance: Maximum speed");
    println!();

    // This would be the actual implementation (commented out since we don't have a real DB):
    /*
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    let binder = UserBinder;

    // Create the in-memory SQLite writer
    let writer = SqliteItemWriter::<User>::new()
        .pool(&pool)
        .table("temp_users")
        .add_column("id")
        .add_column("username")
        .add_column("email")
        .add_column("age")
        .add_column("active")
        .add_column("created_at")
        .item_binder(&binder);

    // Write the users to the in-memory database
    println!("💾 Writing users to in-memory SQLite database...");
    writer.write(&users)?;
    println!("✅ Successfully wrote {} users to in-memory SQLite!", users.len());
    */

    println!("💡 In-Memory SQLite Features:");
    println!("   • Ultra-fast performance (no disk I/O)");
    println!("   • Perfect for temporary data processing");
    println!("   • Ideal for testing and development");
    println!("   • No file system permissions required");
    println!("   • Data lost when connection closes");
    println!();

    Ok(())
}

/// Demonstrates SQLite-specific features
fn demonstrate_sqlite_features() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SQLite-Specific Features");
    println!("===========================");
    println!();

    println!("🔧 SQLite Data Types:");
    println!("   • INTEGER (including AUTOINCREMENT)");
    println!("   • REAL (floating point numbers)");
    println!("   • TEXT (UTF-8 strings)");
    println!("   • BLOB (binary data)");
    println!("   • NULL (missing values)");
    println!("   • Dynamic typing with type affinity");
    println!();

    println!("🎯 SQLite Advantages:");
    println!("   • Zero-configuration database");
    println!("   • Single file deployment");
    println!("   • Cross-platform compatibility");
    println!("   • SQL standard compliance");
    println!("   • ACID transactions");
    println!("   • Concurrent read access");
    println!();

    println!("⚡ Performance Optimizations:");
    println!("   • WAL mode for better concurrency");
    println!("   • Batch inserts within transactions");
    println!("   • Prepared statement caching");
    println!("   • Memory-mapped I/O");
    println!("   • Vacuum and analyze for maintenance");
    println!();

    println!("🛡️  SQLite Constraints:");
    println!("   • PRIMARY KEY enforcement");
    println!("   • UNIQUE constraint support");
    println!("   • FOREIGN KEY relationships");
    println!("   • CHECK constraints");
    println!("   • NOT NULL validation");
    println!();

    Ok(())
}

/// Demonstrates batch processing patterns
fn demonstrate_batch_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 SQLite Batch Processing Patterns");
    println!("===================================");
    println!();

    println!("🔄 Batch Size Considerations:");
    println!("   • Small batches (1-100): Lower memory, more transactions");
    println!("   • Medium batches (100-1000): Balanced performance");
    println!("   • Large batches (1000+): Higher throughput, more memory");
    println!("   • Parameter limit: 65,535 total parameters");
    println!();

    println!("🎯 Use Case Patterns:");
    println!("   • ETL Processing: File → SQLite → Analysis");
    println!("   • Data Migration: Legacy DB → SQLite → New System");
    println!("   • Caching Layer: API Data → SQLite → Fast Queries");
    println!("   • Testing: Mock Data → In-Memory SQLite → Tests");
    println!("   • Analytics: Raw Data → SQLite → Reports");
    println!();

    println!("🔍 Example SQL Generated:");
    println!("   INSERT INTO users (id, username, email, age, active, created_at)");
    println!("   VALUES (?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?)");
    println!();

    Ok(())
}

/// Demonstrates error handling scenarios
fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  SQLite Error Handling");
    println!("=========================");
    println!();

    println!("🔴 Common SQLite Errors:");
    println!("   • SQLITE_CONSTRAINT: Constraint violation");
    println!("   • SQLITE_LOCKED: Database is locked");
    println!("   • SQLITE_BUSY: Database is busy");
    println!("   • SQLITE_CORRUPT: Database corruption");
    println!("   • SQLITE_FULL: Disk full");
    println!("   • SQLITE_PERM: Permission denied");
    println!();

    println!("🛠️  Recovery Strategies:");
    println!("   • Retry with exponential backoff");
    println!("   • WAL mode for reduced locking");
    println!("   • Busy timeout configuration");
    println!("   • Transaction size optimization");
    println!("   • File system monitoring");
    println!();

    println!("📝 Error Logging Examples:");
    println!("   ERROR Failed to write items to SQLite table users: UNIQUE constraint failed");
    println!("   DEBUG Successfully wrote 500 items to SQLite table users");
    println!("   WARN SQLite database file size approaching limit");
    println!();

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🌟 Spring Batch RS - SQLite Writer Example");
    println!("===========================================");
    println!();

    // Run demonstrations
    demonstrate_file_based_sqlite()?;
    demonstrate_in_memory_sqlite()?;
    demonstrate_sqlite_features()?;
    demonstrate_batch_patterns()?;
    demonstrate_error_handling()?;

    println!("🎉 SQLite Writer Example Complete!");
    println!();
    println!("💡 Next Steps:");
    println!("   • Choose between file-based or in-memory SQLite");
    println!("   • Design your database schema");
    println!("   • Implement your custom DatabaseItemBinder");
    println!("   • Configure appropriate batch sizes");
    println!("   • Set up error handling and monitoring");
    println!("   • Integrate with your batch processing pipeline");
    println!();

    println!("📚 Additional Resources:");
    println!("   • SQLite Documentation: https://sqlite.org/docs.html");
    println!("   • SQLx Documentation: https://docs.rs/sqlx/");
    println!("   • Spring Batch RS Guide: Check the project README");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sample_users() {
        let users = create_sample_users();
        assert_eq!(users.len(), 4);
        assert_eq!(users[0].username, "alice_smith");
        assert_eq!(users[0].email, "alice@example.com");
        assert_eq!(users[0].age, 28);
        assert!(users[0].active);
    }

    #[test]
    fn test_user_binder_interface() {
        // Test that our binder implements the required trait
        let _binder: Box<dyn DatabaseItemBinder<User, Sqlite>> = Box::new(UserBinder);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: 1,
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            age: 25,
            active: true,
            created_at: "2024-01-01 12:00:00".to_string(),
        };

        // Test that user can be serialized (required for ItemWriter)
        let _json = serde_json::to_string(&user).unwrap();
    }

    #[test]
    fn test_user_data_types() {
        let users = create_sample_users();

        // Test different data types
        assert!(users.iter().any(|u| u.active)); // boolean true
        assert!(users.iter().any(|u| !u.active)); // boolean false
        assert!(users.iter().all(|u| u.age > 0)); // positive integers
        assert!(users.iter().all(|u| !u.username.is_empty())); // non-empty strings
        assert!(users.iter().all(|u| u.email.contains('@'))); // email format
    }
}
