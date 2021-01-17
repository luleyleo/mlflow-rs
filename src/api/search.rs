use crate::api::run::{Run, RunInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageToken(String);
impl AsRef<str> for PageToken {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
impl From<String> for PageToken {
    fn from(id: String) -> Self {
        PageToken(id)
    }
}
impl From<&str> for PageToken {
    fn from(id: &str) -> Self {
        PageToken(id.to_owned())
    }
}

#[derive(Deserialize)]
pub struct Search {
    pub runs: Vec<Run>,
    pub next_page_token: PageToken,
}

pub struct RunList {
    pub runs: Vec<RunInfo>,
    pub page_token: PageToken,
}
