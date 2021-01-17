use crate::api::{error::*, experiment::*, id::*, run::*, search::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ViewType {
    #[serde(rename = "ACTIVE_ONLY")]
    Active,
    #[serde(rename = "DELETED_ONLY")]
    Deleted,
    #[serde(rename = "ALL")]
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleStage {
    Active,
    Deleted,
}

#[rustfmt::skip]
pub trait Client {
    fn create_experiment(&mut self, name: &str) -> Result<ExperimentId, CreateError>;
    fn list_experiments(&mut self, view_type: ViewType) -> Result<Vec<Experiment>, StorageError>;
    fn get_experiment(&mut self, id: &ExperimentId) -> Result<Experiment, GetError>;
    fn get_experiment_by_name(&mut self, name: &str) -> Result<Experiment, GetError>;
    fn delete_experiment(&mut self, id: &ExperimentId) -> Result<(), DeleteError>;
    fn update_experiment(&mut self, id: &ExperimentId, new_name: Option<&str>) -> Result<(), StorageError>;

    fn create_run(&mut self, experiment: &ExperimentId, start_time: i64, tags: &[RunTag]) -> Result<Run, StorageError>;
    fn delete_run(&mut self, id: &RunId) -> Result<(), DeleteError>;
    fn get_run(&mut self, id: &RunId) -> Result<Run, GetError>;
    fn update_run(&mut self, id: &RunId, status: RunStatus, end_time: i64) -> Result<RunInfo, UpdateError>;
    fn search_runs(&mut self, experiment_ids: &[&ExperimentId], filter: &str, run_view_type: ViewType, max_results: i32, order_by: Option<&str>, page_token: Option<&str>) -> Result<Search, StorageError>;
    fn list_run_infos(&mut self, experiment: &ExperimentId, run_view_type: ViewType, max_results: i32, order_by: Option<&str>, page_token: Option<&str>) -> Result<RunList, StorageError>;
    fn get_metric_history(&mut self, run: &RunId, metric: &str) -> Result<Vec<Metric>, GetError>;

    fn log_param(&mut self, run: &RunId, key: &str, value: &str) -> Result<(), StorageError>;
    fn log_metric(&mut self, run: &RunId, key: &str, value: f64, timestamp: i64, step: i64) -> Result<(), StorageError>;
    fn log_batch(&mut self, run: &RunId, metrics: &[Metric], params: &[Param], tags: &[RunTag]) -> Result<(), BatchError>;
}
