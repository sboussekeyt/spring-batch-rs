use serde::{Deserialize, Serialize};
use spring_batch_rs::item::rdbc::DatabaseItemBinder;
use sqlx::{query_builder::Separated, FromRow, Sqlite};

/// SQLite-specific item binder for Car items.
///
/// This binder handles the conversion of Car domain objects to SQLite-compatible
/// query parameters. It binds values in the order matching the table schema:
/// year (INTEGER), make (VARCHAR), model (VARCHAR), description (VARCHAR).
#[allow(dead_code)]
pub struct SqliteCarItemBinder;

impl DatabaseItemBinder<Car, Sqlite> for SqliteCarItemBinder {
    fn bind(&self, item: &Car, mut query_builder: Separated<Sqlite, &str>) {
        query_builder.push_bind(item.year);
        query_builder.push_bind(item.make.clone());
        query_builder.push_bind(item.model.clone());
        query_builder.push_bind(item.description.clone());
    }
}

/// Car domain model for database operations.
#[derive(Deserialize, Serialize, Debug, Clone, FromRow, PartialEq)]
pub struct Car {
    pub year: i16,
    pub make: String,
    pub model: String,
    pub description: String,
}

impl Car {
    /// Creates a new Car instance.
    pub fn new(year: i16, make: String, model: String, description: String) -> Self {
        Self {
            year,
            make,
            model,
            description,
        }
    }
}

/// SQL statement to create the cars table in SQLite.
#[allow(dead_code)]
pub const CREATE_CARS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS cars (
        year INTEGER NOT NULL,
        make VARCHAR(25) NOT NULL,
        model VARCHAR(25) NOT NULL,
        description VARCHAR(25) NOT NULL
    );";

/// SQL statement to select all cars from the table.
#[allow(dead_code)]
pub const SELECT_ALL_CARS_SQL: &str = "SELECT year, make, model, description FROM cars";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_creation() {
        let car = Car::new(
            1967,
            "Ford".to_string(),
            "Mustang fastback 1967".to_string(),
            "American car".to_string(),
        );

        assert_eq!(car.year, 1967);
        assert_eq!(car.make, "Ford");
        assert_eq!(car.model, "Mustang fastback 1967");
        assert_eq!(car.description, "American car");
    }
}
