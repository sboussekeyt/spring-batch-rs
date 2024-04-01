/// The `json_reader` module contains the `JsonItemReader` struct, which is the main entry point for reading items from a JSON data source.
/// It implements the `ItemReader` trait and provides methods for reading items from a JSON data source and deserializing them into Rust structs.
///
pub mod json_reader;
/// The `json_writer` module contains the `JsonItemWriter` struct, which is the main entry point for writing items to a JSON data source.
/// It implements the `ItemWriter` trait and provides methods for serializing Rust structs into JSON and writing them to a data source.
///
pub mod json_writer;
