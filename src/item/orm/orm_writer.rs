use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult};
use std::marker::PhantomData;

use crate::{
    core::item::{ItemWriter, ItemWriterResult},
    BatchError,
};

/// A writer for writing ORM active models directly to a database.
///
/// This writer provides an implementation of the `ItemWriter` trait for ORM-based
/// database operations. It works directly with ORM active models, eliminating the
/// need for mapper layers and providing a simple, efficient interface for batch
/// database operations.
///
/// # Design Philosophy
///
/// The writer follows the "direct entity" approach used throughout the Spring Batch RS
/// ORM integration:
/// - **No Mappers**: Works directly with ORM active models, no transformation layer
/// - **Type Safety**: Leverages ORM's compile-time type safety
/// - **Efficiency**: Direct operations without intermediate conversions
/// - **Simplicity**: Clean API with minimal configuration required
///
/// # Trait Bounds Design
///
/// The writer requires `A: ActiveModelTrait + Send` where:
/// - `ActiveModelTrait` provides the associated Entity type and database operations
/// - `Send` enables safe transfer across async boundaries
/// - The Entity type is automatically inferred from `<A as ActiveModelTrait>::Entity`
///
/// Note: `IntoActiveModel<A>` is automatically provided by SeaORM's blanket implementation
/// for all types that implement `ActiveModelTrait`, so it's not explicitly required.
///
/// # Usage Pattern
///
/// Users should convert their business objects to ORM active models before writing,
/// either manually or using processors in the batch pipeline. This approach provides
/// maximum flexibility and performance.
///
/// # Database Operations
///
/// The writer uses ORM's built-in batch insert capabilities:
/// - **Connection Management**: Uses ORM's connection management for database operations
/// - **Batch Operations**: Performs batch inserts to minimize database round trips
/// - **Transaction Support**: Leverages ORM's transaction handling for consistency
/// - **Type Safety**: Leverages ORM's type-safe active model operations
///
/// # Thread Safety
///
/// This writer is **not thread-safe** as it's designed for single-threaded batch processing
/// scenarios. If you need concurrent access, consider using multiple writer instances.
///
/// # Database Support
///
/// This writer supports all databases that SeaORM supports:
/// - PostgreSQL
/// - MySQL
/// - SQLite
/// - SQL Server (limited support)
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::orm::{OrmItemWriter, OrmItemWriterBuilder};
/// use spring_batch_rs::core::item::ItemWriter;
/// use sea_orm::{Database, ActiveValue::Set};
/// use serde::{Deserialize, Serialize};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create database connection
/// let db = Database::connect("sqlite::memory:").await?;
///
/// // Create the writer with single type parameter (just the ActiveModel)
/// // let writer: OrmItemWriter<product::ActiveModel> = OrmItemWriterBuilder::new()
/// //     .connection(&db)
/// //     .build();
///
/// // Work directly with ORM active models
/// // let active_models = vec![
/// //     product::ActiveModel {
/// //         name: Set("Laptop".to_string()),
/// //         category: Set("Electronics".to_string()),
/// //         price: Set(999.99),
/// //         in_stock: Set(true),
/// //         ..Default::default()
/// //     },
/// // ];
/// // writer.write(&active_models)?;
/// # Ok(())
/// # }
/// ```
pub struct OrmItemWriter<'a, O>
where
    O: ActiveModelTrait + Send,
{
    /// Database connection reference
    /// This ensures the connection remains valid throughout the writer's lifecycle
    connection: &'a DatabaseConnection,
    /// Phantom data to track the active model type
    _phantom: PhantomData<O>,
}

impl<'a, O> OrmItemWriter<'a, O>
where
    O: ActiveModelTrait + Send,
{
    /// Creates a new ORM item writer.
    ///
    /// # Parameters
    /// - `connection`: Database connection reference
    ///
    /// # Returns
    /// A new ORM item writer instance
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::orm::OrmItemWriter;
    /// use sea_orm::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::connect("sqlite::memory:").await?;
    ///
    /// // Create writer for your ORM active model type (Entity is inferred)
    /// // let writer = OrmItemWriter::<product::ActiveModel>::new(&db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(connection: &'a DatabaseConnection) -> Self {
        Self {
            connection,
            _phantom: PhantomData,
        }
    }

    /// Performs the actual database insert operation asynchronously.
    ///
    /// This method converts the runtime to handle async operations within
    /// the synchronous ItemWriter interface.
    ///
    /// # Parameters
    /// - `active_models`: Vector of active models to insert
    ///
    /// # Returns
    /// - `Ok(InsertResult)` if the insert operation succeeds
    /// - `Err(DbErr)` if the database operation fails
    async fn insert_batch_async(&self, active_models: Vec<O>) -> Result<InsertResult<O>, DbErr> {
        <O as ActiveModelTrait>::Entity::insert_many(active_models)
            .exec(self.connection)
            .await
    }

    /// Performs a batch insert operation.
    ///
    /// This method handles the conversion between sync and async contexts
    /// using tokio's block_in_place to avoid blocking the async runtime.
    ///
    /// # Parameters
    /// - `active_models`: Vector of active models to insert
    ///
    /// # Returns
    /// - `Ok(())` if the insert operation succeeds
    /// - `Err(BatchError)` if the operation fails
    fn insert_batch(&self, active_models: Vec<O>) -> Result<(), BatchError> {
        // Use tokio's block_in_place to handle async operations in sync context
        // This is the same pattern used in the ORM reader
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.insert_batch_async(active_models).await })
        });

        match result {
            Ok(_insert_result) => {
                log::debug!("Successfully inserted batch to database");
                Ok(())
            }
            Err(db_err) => {
                let error_msg = format!("Failed to insert batch to database: {}", db_err);
                log::error!("{}", error_msg);
                Err(BatchError::ItemWriter(error_msg))
            }
        }
    }
}

impl<O> ItemWriter<O> for OrmItemWriter<'_, O>
where
    O: ActiveModelTrait + Send,
{
    /// Writes ORM active models directly to the database.
    ///
    /// This method performs batch insert operations for efficiency, writing all
    /// active models in a single database operation when possible.
    ///
    /// # Process Flow
    ///
    /// 1. **Validation**: Check if there are items to write
    /// 2. **Batch Insert**: Perform a single batch insert operation
    /// 3. **Error Handling**: Convert any database errors to BatchError
    ///
    /// # Parameters
    /// - `items`: A slice of ORM active models to write to the database
    ///
    /// # Returns
    /// - `Ok(())` if all items are successfully written
    /// - `Err(BatchError)` if any error occurs during the process
    ///
    /// # Database Operations
    ///
    /// The method uses ORM's `insert_many()` function, which:
    /// - Generates a single INSERT statement with multiple VALUE clauses
    /// - Minimizes database round trips for better performance
    /// - Maintains transactional consistency for the entire batch
    /// - Returns the number of affected rows
    ///
    /// # Error Handling
    ///
    /// Errors can occur during database operations such as:
    /// - Constraint violations (unique, foreign key, etc.)
    /// - Connection failures
    /// - Invalid data types or values
    ///
    /// All errors are converted to `BatchError::ItemWriter` with descriptive messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::orm::{OrmItemWriter, OrmItemWriterBuilder};
    /// use spring_batch_rs::core::item::ItemWriter;
    /// use sea_orm::{Database, ActiveValue::Set};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::connect("sqlite::memory:").await?;
    ///
    /// // let writer: OrmItemWriter<user::ActiveModel> = OrmItemWriterBuilder::new()
    /// //     .connection(&db)
    /// //     .build();
    ///
    /// // Write ORM active models directly
    /// // let active_models = vec![
    /// //     user::ActiveModel {
    /// //         name: Set("Alice".to_string()),
    /// //         email: Set("alice@example.com".to_string()),
    /// //         ..Default::default()
    /// //     },
    /// //     user::ActiveModel {
    /// //         name: Set("Bob".to_string()),
    /// //         email: Set("bob@example.com".to_string()),
    /// //         ..Default::default()
    /// //     },
    /// // ];
    /// // writer.write(&active_models)?;
    /// # Ok(())
    /// # }
    /// ```
    fn write(&self, items: &[O]) -> ItemWriterResult {
        log::debug!("Writing {} active models to database", items.len());

        if items.is_empty() {
            log::debug!("No items to write, skipping database operation");
            return Ok(());
        }

        // Clone all active models for the batch insert
        let active_models: Vec<O> = items.to_vec();

        // Perform batch insert
        self.insert_batch(active_models)?;

        log::info!(
            "Successfully wrote {} active models to database",
            items.len()
        );
        Ok(())
    }

    /// Flushes any pending operations.
    ///
    /// For the ORM writer, this is a no-op since each write operation
    /// immediately commits to the database. There are no pending operations
    /// to flush.
    ///
    /// # Returns
    /// Always returns `Ok(())`
    fn flush(&self) -> ItemWriterResult {
        log::debug!("Flush called on ORM writer (no-op)");
        Ok(())
    }

    /// Opens the writer for writing.
    ///
    /// For the ORM writer, this is a no-op since ORM manages
    /// database connections internally and no special initialization is required.
    ///
    /// # Returns
    /// Always returns `Ok(())`
    fn open(&self) -> ItemWriterResult {
        log::debug!("Opened ORM writer");
        Ok(())
    }

    /// Closes the writer and releases any resources.
    ///
    /// For the ORM writer, this is a no-op since ORM manages
    /// database connections internally and no special cleanup is required.
    ///
    /// # Returns
    /// Always returns `Ok(())`
    fn close(&self) -> ItemWriterResult {
        log::debug!("Closed ORM writer");
        Ok(())
    }
}

/// A builder for creating ORM item writers.
///
/// This builder allows you to configure an ORM item writer with the necessary
/// database connection. Since the writer now works directly with ORM active models,
/// no mapper configuration is required.
///
/// # Design Pattern
///
/// This struct implements the Builder pattern, which allows for fluent, chainable
/// configuration of an `OrmItemWriter` before creation. The simplified design
/// requires only a database connection and infers the Entity type from the
/// ActiveModel's associated type.
///
/// # Required Configuration
///
/// The following parameter is required and must be set before calling `build()`:
/// - **Connection**: Database connection reference
///
/// # Examples
///
/// ```
/// use spring_batch_rs::item::orm::OrmItemWriterBuilder;
/// use sea_orm::Database;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = Database::connect("sqlite::memory:").await?;
///
/// // Only need to specify the ActiveModel type - Entity is inferred!
/// // let builder: OrmItemWriterBuilder<product::ActiveModel> = OrmItemWriterBuilder::new()
/// //     .connection(&db);
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct OrmItemWriterBuilder<'a, O>
where
    O: ActiveModelTrait + Send,
{
    /// Database connection - None until set by the user
    /// This will be validated as required during build()
    connection: Option<&'a DatabaseConnection>,
    /// Phantom data to track the active model type
    _phantom: PhantomData<O>,
}

impl<'a, O> OrmItemWriterBuilder<'a, O>
where
    O: ActiveModelTrait + Send,
{
    /// Creates a new ORM item writer builder.
    ///
    /// All configuration options start as None and must be set before calling `build()`.
    ///
    /// # Returns
    /// A new builder instance with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::orm::OrmItemWriterBuilder;
    ///
    /// // Create a new builder (only need ActiveModel type!)
    /// // let builder = OrmItemWriterBuilder::<MyActiveModel>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            connection: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the database connection for the item writer.
    ///
    /// This parameter is **required**. The builder will panic during `build()`
    /// if this parameter is not set.
    ///
    /// # Parameters
    /// - `connection`: Reference to the ORM database connection
    ///
    /// # Returns
    /// The updated builder instance for method chaining
    ///
    /// # Connection Lifecycle
    ///
    /// The connection reference must remain valid for the entire lifetime of the
    /// resulting writer. The writer does not take ownership of the connection,
    /// allowing it to be shared across multiple components.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::orm::OrmItemWriterBuilder;
    /// use sea_orm::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::connect("sqlite::memory:").await?;
    ///
    /// // Much cleaner with single generic parameter!
    /// // let builder: OrmItemWriterBuilder<product::ActiveModel> = OrmItemWriterBuilder::new()
    /// //     .connection(&db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn connection(mut self, connection: &'a DatabaseConnection) -> Self {
        self.connection = Some(connection);
        self
    }

    /// Builds the ORM item writer with the configured parameters.
    ///
    /// This method validates that all required parameters have been set and creates
    /// a new `OrmItemWriter` instance.
    ///
    /// # Returns
    /// A configured `OrmItemWriter` instance
    ///
    /// # Panics
    /// Panics if the required database connection parameter is missing.
    ///
    /// # Validation
    ///
    /// The builder performs the following validation:
    /// - Ensures a database connection has been provided
    ///
    /// If any validation fails, the method will panic with a descriptive error message.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::orm::OrmItemWriterBuilder;
    /// use sea_orm::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::connect("sqlite::memory:").await?;
    ///
    /// // let writer: OrmItemWriter<product::ActiveModel> = OrmItemWriterBuilder::new()
    /// //     .connection(&db)
    /// //     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> OrmItemWriter<'a, O> {
        let connection = self
            .connection
            .expect("Database connection is required. Call .connection() before .build()");

        OrmItemWriter::new(connection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{
        entity::prelude::*,
        ActiveValue::{NotSet, Set},
    };

    // Mock entity and active model for testing trait bounds
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "test_entity")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[test]
    fn test_simplified_trait_bounds_compilation() {
        // This test verifies that our simplified trait bounds compile correctly
        // with only one generic parameter (ActiveModel)
        // If this compiles, it means:
        // 1. ActiveModelTrait + Send is sufficient
        // 2. Entity type can be inferred from <A as ActiveModelTrait>::Entity

        // Test that we can specify trait bounds with just ActiveModel
        fn _verify_bounds<A>()
        where
            A: ActiveModelTrait + Send,
        {
            // This function will only compile if our trait bounds are sufficient
            // for ORM operations
        }

        // Verify that our actual types satisfy the bounds
        _verify_bounds::<ActiveModel>();

        // Test that we can create a builder with just ActiveModel
        let _builder = OrmItemWriterBuilder::<ActiveModel>::new();

        // Test that the builder has the correct type signature
        assert!(_builder.connection.is_none());
    }

    #[test]
    fn test_active_model_creation() {
        // Test that we can create active models that satisfy our trait bounds
        let active_model = ActiveModel {
            id: NotSet,
            name: Set("Test".to_owned()),
        };

        // Verify that ActiveModel implements the required traits
        fn check_traits<A>(_: A)
        where
            A: ActiveModelTrait + Send,
        {
            // This function will only compile if A satisfies our trait bounds
        }

        check_traits(active_model);
    }

    #[test]
    fn test_entity_inference() {
        // Verify that we can infer the Entity type from ActiveModel
        fn check_entity_inference<A>()
        where
            A: ActiveModelTrait + Send,
            <A as ActiveModelTrait>::Entity: EntityTrait,
        {
            // This function will only compile if we can access the Entity type
            // through the ActiveModel's associated type
        }

        check_entity_inference::<ActiveModel>();
    }

    #[test]
    fn test_simplified_builder_pattern() {
        // This test demonstrates that the builder pattern works with our simplified bounds
        // Note: We can't actually build without a real database connection,
        // but we can test that the types compile correctly

        let builder = OrmItemWriterBuilder::<ActiveModel>::new();

        // The builder should have the correct type with only one generic parameter
        assert!(builder.connection.is_none());
    }
}
