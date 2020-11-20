use nanoserde::{DeJson, SerJson};

type Int64 = i64;

#[derive(Debug, SerJson, DeJson)]
pub struct Experiment {
    pub experiment_id: String,
    pub name: String,
    pub artifact_location: String,
    pub lifecycle_stage: String,
    pub last_update_time: Int64,
    pub creation_time: Int64,
    pub tags: Vec<ExperimentTag>,
}

#[derive(Debug, SerJson, DeJson)]
pub struct ExperimentTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug, SerJson, DeJson)]
pub struct FileInfo {
    pub path: String,
    pub is_dir: bool,
    pub file_size: Int64,
}

#[derive(Debug, SerJson, DeJson)]
pub struct Metric {
    pub key: String,
    pub value: f64,
    pub timestamp: Int64,
    pub step: Int64,
}

#[derive(Debug, SerJson, DeJson)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Debug, SerJson, DeJson)]
pub struct Run {
    pub info: RunInfo,
    pub data: RunData,
}

#[derive(Debug, SerJson, DeJson)]
pub struct RunData {
    pub metrics: Vec<Metric>,
    pub params: Vec<Param>,
    pub tags: Vec<RunTag>,
}

#[derive(Debug, SerJson, DeJson)]
pub struct RunInfo {
    pub run_id: String,
    #[deprecated = "This field will be removed in a future FLflow version"]
    pub run_uuid: String,
    pub experiment_id: String,
    #[deprecated = "This field will be removed in a future FLflow version"]
    pub user_id: String,
    pub status: String,
    pub start_time: Int64,
    pub end_time: Int64,
    pub artifact_uri: String,
    pub lifecycle_stage: String,
}

#[derive(Debug, SerJson, DeJson)]
pub struct RunTag {
    pub key: String,
    pub value: String,
}