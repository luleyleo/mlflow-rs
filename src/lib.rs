//mod client;
//mod experiment;
//mod storage;

pub mod api;
//pub mod tracking;
pub mod backend;

//pub use client::Client;
//pub use experiment::Experiment;

/// All the errors.
//pub use storage::errors;

pub use api::id::{ExperimentId, RunId};

/// Utility function to create a MLflow timestamp.
pub fn timestamp() -> u64 {
    use std::convert::TryInto;
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}
