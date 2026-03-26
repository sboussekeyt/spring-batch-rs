use serde::{Deserialize, Serialize};
use spring_batch_rs::item::rdbc::DatabaseItemBinder;
use sqlx::{query_builder::Separated, FromRow, Postgres};

/// PostgreSQL-specific item binder for Car items.
///
/// This binder handles the conversion of Car domain objects to PostgreSQL-compatible
/// query parameters. It binds values in the order matching the table schema:
/// year (SMALLINT), make (TEXT), model (TEXT), description (TEXT).
#[allow(dead_code)]
pub struct PostgresCarItemBinder;

impl DatabaseItemBinder<Car, Postgres> for PostgresCarItemBinder {
    fn bind(&self, item: &Car, mut query_builder: Separated<Postgres, &str>) {
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

/// SQL statement to create the cars table in PostgreSQL.
#[allow(dead_code)]
pub const CREATE_CARS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS cars (
        year SMALLINT NOT NULL,
        make TEXT NOT NULL,
        model TEXT NOT NULL,
        description TEXT NOT NULL
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
            1948,
            "Porsche".to_string(),
            "356".to_string(),
            "Luxury sports car".to_string(),
        );

        assert_eq!(car.year, 1948);
        assert_eq!(car.make, "Porsche");
        assert_eq!(car.model, "356");
        assert_eq!(car.description, "Luxury sports car");
    }
}
