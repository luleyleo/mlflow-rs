use serde::{Deserialize, Serialize};

type Int64 = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Experiment {
    pub experiment_id: String,
    pub name: String,
    pub artifact_location: String,
    pub lifecycle_stage: String,
    pub last_update_time: Option<Int64>,
    pub creation_time: Option<Int64>,
    pub tags: Option<Vec<ExperimentTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExperimentTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub is_dir: bool,
    pub file_size: Int64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metric {
    pub key: String,
    pub value: f64,
    pub timestamp: Int64,
    pub step: Int64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub info: RunInfo,
    pub data: RunData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunData {
    pub metrics: Option<Vec<Metric>>,
    pub params: Option<Vec<Param>>,
    pub tags: Option<Vec<RunTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunInfo {
    pub run_id: String,
    #[deprecated = "This field will be removed in a future FLflow version"]
    pub run_uuid: String,
    pub experiment_id: String,
    #[deprecated = "This field will be removed in a future FLflow version"]
    pub user_id: String,
    pub status: String,
    pub start_time: String,
    pub end_time: Option<Int64>,
    pub artifact_uri: String,
    pub lifecycle_stage: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunTag {
    pub key: String,
    pub value: String,
}
