use crate::{Experiment, Id, Run, run::RunStatus};

mod server;
pub(crate) use server::Storage as Server;

pub(crate) trait Storage {
    fn create_experiment(&self, name: &str) -> Experiment;
    fn list_experiments(&self) -> Vec<Experiment>;
    fn get_experiment(&self, name: &str) -> Option<Experiment>;

    fn create_run(&self, experiment: &Id, start_time: u64) -> Run;
    fn terminate_run(&self, run: &Id, status: RunStatus, end_time: u64);

    fn log_param(&self, run: &Id, key: &str, value: &str);
    fn log_metric(&self, run: &Id, key: &str, value: f64, time_stamp: u64, step: u64);
}