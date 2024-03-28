use rand::distributions::{Alphanumeric, DistString};

pub mod item;
pub mod job;
pub mod step;

fn build_name() -> String {
    Alphanumeric
        .sample_string(&mut rand::thread_rng(), 8)
        .clone()
}
