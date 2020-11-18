use crate::{Experiment, Id};

pub struct Storage {
    url: String
}

impl Storage {
    pub fn new(url: &str) -> Self {
        Storage {
            url: url.to_string()
        }
    }
}

impl super::Storage for Storage {
    fn create_experiment(&self, name: &str) -> Experiment {
        todo!()
    }

    fn get_experiment(&self, name: &str) -> Option<Experiment> {
        todo!()
    }

    fn list_experiments(&self) -> Vec<crate::Experiment> {
        todo!()
    }

    fn create_run(&self, experiment: &Id, start_time: u64) -> crate::Run {
        todo!()
    }

    fn terminate_run(&self, run: &Id, status: crate::run::RunStatus, end_time: u64) {
        todo!()
    }

    fn log_param(&self, run: &Id, key: &str, value: &str) {
        todo!()
    }

    fn log_metric(&self, run: &Id, key: &str, value: f64, time_stamp: u64, step: u64) {
        todo!()
    }
}
