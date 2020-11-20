use crate::{Experiment, Id, storage::{errors::StorageError, primitive}};

pub struct Run<'a> {
    experiment: &'a Experiment<'a>,
    id: Id,
}

pub enum RunStatus {
    Running,
    Scheduled,
    Finished,
    Failed,
    Killed,
}

impl<S: AsRef<str>> From<S> for RunStatus {
    fn from(string: S) -> Self {
        match string.as_ref() {
            "RUNNING" => RunStatus::Running,
            "SCHEDULED" => RunStatus::Scheduled,
            "FINISHED" => RunStatus::Finished,
            "FAILED" => RunStatus::Failed,
            "KILLED" => RunStatus::Killed,
            s => panic!("Unknown run status {}", s),
        }
    }
}

impl<'a> Run<'a> {
    pub(crate) fn new(experiment: &'a Experiment, run: primitive::Run) -> Self {
        Run {
            experiment,
            id: run.info.run_id,
        }
    }

    pub fn log_param(&self, name: &str, value: &str) {
        self.try_log_param(name, value).unwrap();
    }
    pub fn try_log_param(&self, name: &str, value: &str) -> Result<(), StorageError> {
        self.experiment.client.storage.log_param(&self.id, name, value)
    }

    pub fn log_metric(&self, name: &str, value: f64, time_stamp: u64, step: u64) {
        self.try_log_metric(name, value, time_stamp, step).unwrap();
    }
    pub fn try_log_metric(&self, name: &str, value: f64, time_stamp: u64, step: u64) -> Result<(), StorageError> {
        self.experiment.client.storage.log_metric(&self.id, name, value, time_stamp, step)
    }

    pub fn terminate(self) {
        self.try_terminate().unwrap()
    }
    pub fn try_terminate(self) -> Result<(), StorageError> {
        let end_time = crate::time_stamp();
        self.experiment.client.storage.terminate_run(&self.id, end_time)
    }
}