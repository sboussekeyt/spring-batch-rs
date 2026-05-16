use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

/// SQL statement to create the cars table in MySQL.
#[allow(dead_code)]
pub const CREATE_CARS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS cars (
        year SMALLINT NOT NULL,
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
            2021,
            "Mazda".to_string(),
            "CX-30".to_string(),
            "SUV Compact".to_string(),
        );

        assert_eq!(car.year, 2021);
        assert_eq!(car.make, "Mazda");
        assert_eq!(car.model, "CX-30");
        assert_eq!(car.description, "SUV Compact");
    }
}
