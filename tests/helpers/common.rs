/// Common test utilities and constants shared across database tests.
/// Sample CSV data for testing car data imports.
pub const SAMPLE_CARS_CSV: &str = "year,make,model,description
1948,Porsche,356,Luxury sports car
2011,Peugeot,206+,City car
2012,Citroën,C4 Picasso,SUV
2021,Mazda,CX-30,SUV Compact
1967,Ford,Mustang fastback 1967,American car";

/// Expected number of cars in the sample CSV data.
pub const EXPECTED_CAR_COUNT: usize = 5;

/// Expected CSV output for person data (18 records).
pub const EXPECTED_PERSON_CSV: &str = "id,first_name,last_name
1,Melton,Finnegan
2,Pruitt,Brayan
3,Simmons,Kaitlyn
4,Dougherty,Kristen
5,Patton,Gina
6,Michael,Emiliano
7,Singh,Zion
8,Morales,Kaydence
9,Hull,Randy
10,Crosby,Daphne
11,Gates,Christopher
12,Colon,Melina
13,Alvarado,Nathan
14,Blackwell,Mareli
15,Lara,Kian
16,Montes,Cory
17,Larson,Iyana
18,Gentry,Sasha
";

/// Expected number of persons in the migration data.
pub const EXPECTED_PERSON_COUNT: usize = 18;

/// Default chunk size for batch processing in tests.
pub const DEFAULT_CHUNK_SIZE: u16 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_csv_line_count() {
        let lines: Vec<&str> = SAMPLE_CARS_CSV.lines().collect();
        // 1 header + 5 data rows
        assert_eq!(lines.len(), 6);
    }

    #[test]
    fn test_expected_person_csv_line_count() {
        let lines: Vec<&str> = EXPECTED_PERSON_CSV.lines().collect();
        // 1 header + 18 data rows
        assert_eq!(lines.len(), 19);
    }
}
