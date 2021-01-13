pub mod api;
//pub mod tracking;
pub mod backend;

pub use api::id::{ExperimentId, RunId};
pub use api::client::Client;

/// Utility function to create a MLflow timestamp.
pub fn timestamp() -> i64 {
    use std::convert::TryInto;
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}
