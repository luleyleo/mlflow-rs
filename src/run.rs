use crate::{Experiment, Id};

pub struct Run<'a> {
    experiment: &'a Experiment<'a>,
    id: Id,
    start_time: u64,
    status: RunStatus,
}

pub enum RunStatus {
    Running,
    Scheduled,
    Finished,
    Failed,
    Killed,
}


impl Run<'_> {
    pub fn log_param(&self, name: &str, value: &str) {
        self.experiment.client.storage.log_param(&self.id, name, value);
    }

    pub fn log_metric(&self, name: &str, value: f64, time_stamp: u64, step: u64) {
        self.experiment.client.storage.log_metric(&self.id, name, value, time_stamp, step);
    }

    pub fn terminate(self) {
        let end_time = crate::time_stamp();
        self.experiment.client.storage.terminate_run(&self.id, RunStatus::Finished, end_time);
    }
}