use crate::storage::{errors::StorageError, primitive, RunStorage};

/// A MLflow Run.
///
/// This can be created using [`Experiment::create_run`].
///
/// It allows logging [parameters][self::Run::log_param()] and [metrics][self::Run::log_metric()].
pub struct Run {
    storage: Box<dyn RunStorage>,
    id: String,
}

impl Run {
    pub(crate) fn new(storage: Box<dyn RunStorage>, run: primitive::Run) -> Self {
        Run {
            storage,
            id: run.info.run_id,
        }
    }
}

/// Run methods without error handling.
impl Run {
    pub fn log_param(&mut self, name: &str, value: &str) {
        self.try_log_param(name, value).unwrap();
    }

    pub fn log_metric(&mut self, name: &'static str, value: f64, time_stamp: u64, step: u64) {
        self.try_log_metric(name, value, time_stamp, step).unwrap();
    }

    pub fn terminate(self) {
        self.try_terminate().unwrap()
    }
}

/// Run methods with error handling.
impl Run {
    pub fn try_log_param(&mut self, name: &str, value: &str) -> Result<(), StorageError> {
        self.storage.log_param(&self.id, name, value)
    }

    pub fn try_log_metric(
        &mut self,
        name: &'static str,
        value: f64,
        time_stamp: u64,
        step: u64,
    ) -> Result<(), StorageError> {
        self.storage
            .log_metric(&self.id, name, value, time_stamp, step)
    }

    pub fn try_terminate(mut self) -> Result<(), StorageError> {
        let end_time = crate::timestamp();
        self.storage.terminate(&self.id, end_time)
    }
}
