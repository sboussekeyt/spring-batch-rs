use rand::distributions::{Alphanumeric, DistString};

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
        .sample_string(&mut rand::thread_rng(), 8)
        .clone()
}
