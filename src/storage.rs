use crate::{Experiment, Run};

pub mod errors;
use errors::{CreateExperimentError, GetExperimentError, StorageError};

mod server;
pub(crate) use server::ServerClientStorage as Server;

pub(crate) mod primitive;

pub(crate) struct BufferedMetric {
    pub name: &'static str,
    pub value: f64,
    pub timestamp: u64,
    pub step: u64,
}

pub(crate) trait ClientStorage {
    fn create_experiment(&mut self, name: &str) -> Result<Experiment, CreateExperimentError>;
    fn list_experiments(&mut self) -> Result<Vec<Experiment>, StorageError>;
    fn get_experiment(&mut self, name: &str) -> Result<Experiment, GetExperimentError>;
}

pub(crate) trait ExperimentStorage {
    fn create_run(&mut self, experiment: &str, start_time: u64) -> Result<Run, StorageError>;
}

pub(crate) trait RunStorage {
    fn log_param(&mut self, run: &str, key: &str, value: &str) -> Result<(), StorageError>;
    fn log_metric(
        &mut self,
        run: &str,
        key: &'static str,
        value: f64,
        timestamp: u64,
        step: u64,
    ) -> Result<(), StorageError>;
    fn terminate(&mut self, run: &str, end_time: u64) -> Result<(), StorageError>;
}

pub(crate) trait Storage {
    fn create_experiment(&self, name: &str)
        -> Result<primitive::Experiment, CreateExperimentError>;
    fn list_experiments(&self) -> Result<Vec<primitive::Experiment>, StorageError>;
    fn get_experiment(&self, name: &str) -> Result<primitive::Experiment, GetExperimentError>;

    fn create_run(&self, experiment: &str, start_time: u64)
        -> Result<primitive::Run, StorageError>;
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
    fn log_batch(&self, run: &str, metrics: &mut Vec<BufferedMetric>) -> Result<(), StorageError>;
}
