use serde::{Serialize, Deserialize};

use crate::ExperimentId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Experiment {
    pub experiment_id: ExperimentId,
    pub name: String,
    pub artifact_location: String,
    pub lifecycle_stage: String,
    pub last_update_time: Option<i64>,
    pub creation_time: Option<i64>,
    pub tags: Option<Vec<ExperimentTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExperimentTag {
    pub key: String,
    pub value: String,
}
