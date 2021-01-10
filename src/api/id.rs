use serde::{Serialize, Deserialize};

// EXPERIMENTS

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExperimentId(String);

impl AsRef<str> for ExperimentId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for ExperimentId {
    fn from(id: String) -> Self {
        ExperimentId(id)
    }
}

impl From<&str> for ExperimentId {
    fn from(id: &str) -> Self {
        ExperimentId(id.to_owned())
    }
}

// RUNS

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(String);

impl AsRef<str> for RunId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for RunId {
    fn from(id: String) -> Self {
        RunId(id)
    }
}

impl From<&str> for RunId {
    fn from(id: &str) -> Self {
        RunId(id.to_owned())
    }
}