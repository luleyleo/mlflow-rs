use crate::{
    storage::{errors::StorageError, primitive, BufferedMetric},
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

pub struct BufferedRun<'a> {
    inner: Run<'a>,
    buffer: Vec<BufferedMetric>,
}

impl<'a> Run<'a> {
    pub(crate) fn new(experiment: &'a Experiment, run: primitive::Run) -> Self {
        Run {
            experiment,
            id: run.info.run_id,
        }
    }

    pub fn buffer(self) -> BufferedRun<'a> {
        BufferedRun::new(self)
    }
}

/// Run methods without error handling.
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

/// Run methods with error handling.
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

impl<'a> BufferedRun<'a> {
    pub fn new(run: Run<'a>) -> Self {
        BufferedRun {
            inner: run,
            buffer: Vec::with_capacity(1000),
        }
    }

    pub fn unwrap(self) -> Run<'a> {
        self.inner
    }
}

impl BufferedRun<'_> {
    pub fn log_metric(&mut self, name: &'static str, value: f64, timestamp: u64, step: u64) {
        self.buffer.push(BufferedMetric {
            name,
            value,
            timestamp,
            step,
        });
    }

    pub fn submit(&mut self) {
        self.try_submit().unwrap();
    }

    pub fn try_submit(&mut self) -> Result<(), StorageError> {
        self.inner
            .experiment
            .client
            .storage
            .log_batch(&self.inner.id, &mut self.buffer)
    }
}
