use crate::{
    storage::{errors::StorageError, primitive},
    Experiment,
};

/// A MLflow Run.
///
/// This can be created using [`Experiment::create_run`].
///
/// It allows logging [parameters][self::Run::log_param()] and [metrics][self::Run::log_metric()].
pub struct Run<'a> {
    experiment: &'a Experiment<'a>,
    id: String,
}

impl<'a> Run<'a> {
    pub(crate) fn new(experiment: &'a Experiment, run: primitive::Run) -> Self {
        Run {
            experiment,
            id: run.info.run_id,
        }
    }
}

/// Client methods without error handling.
impl Run<'_> {
    pub fn log_param(&self, name: &str, value: &str) {
        self.try_log_param(name, value).unwrap();
    }

    pub fn log_metric(&self, name: &str, value: f64, time_stamp: u64, step: u64) {
        self.try_log_metric(name, value, time_stamp, step).unwrap();
    }

    pub fn terminate(self) {
        self.try_terminate().unwrap()
    }
}

/// Client methods with error handling.
impl Run<'_> {
    pub fn try_log_param(&self, name: &str, value: &str) -> Result<(), StorageError> {
        self.experiment
            .client
            .storage
            .log_param(&self.id, name, value)
    }

    pub fn try_log_metric(
        &self,
        name: &str,
        value: f64,
        time_stamp: u64,
        step: u64,
    ) -> Result<(), StorageError> {
        self.experiment
            .client
            .storage
            .log_metric(&self.id, name, value, time_stamp, step)
    }

    pub fn try_terminate(self) -> Result<(), StorageError> {
        let end_time = crate::timestamp();
        self.experiment
            .client
            .storage
            .terminate_run(&self.id, end_time)
    }
}
