use rand::distr::{Alphanumeric, SampleString};

pub mod item;

pub mod job;

pub mod step;

/// Generates a random name consisting of alphanumeric characters.
///
/// # Returns
///
/// A `String` containing the generated random name.
fn build_name() -> String {
    Alphanumeric
        .sample_string(&mut rand::rng(), 8)
        .clone()
}
