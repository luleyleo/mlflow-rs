use crate::storage::primitive::{Experiment, Run};

pub mod errors;
use errors::{CreateExperimentError, GetExperimentError, StorageError};

mod server;
pub(crate) use server::Storage as Server;

pub(crate) mod primitive;

pub(crate) trait Storage {
    fn create_experiment(&self, name: &str) -> Result<Experiment, CreateExperimentError>;
    fn list_experiments(&self) -> Result<Vec<Experiment>, StorageError>;
    fn get_experiment(&self, name: &str) -> Result<Experiment, GetExperimentError>;

    fn create_run(&self, experiment: &str, start_time: u64) -> Result<Run, StorageError>;
    fn terminate_run(&self, run: &str, end_time: u64) -> Result<(), StorageError>;

    fn log_param(&self, run: &str, key: &str, value: &str) -> Result<(), StorageError>;
    fn log_metric(
        &self,
        run: &str,
        key: &str,
        value: f64,
        time_stamp: u64,
        step: u64,
    ) -> Result<(), StorageError>;
}
