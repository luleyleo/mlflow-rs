mod client;
mod experiment;
mod run;
mod storage;

pub use client::Client;
pub use experiment::Experiment;
pub use run::Run;

/// All the errors.
pub use storage::errors;

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
