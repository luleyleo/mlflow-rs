use crate::{Client, Id, Run};

pub struct Experiment<'a> {
    pub(crate) client: &'a Client,
    id: Id,
    name: String,
}

impl Experiment<'_> {
    pub fn create_run(&self) -> Run {
        let start_time = crate::time_stamp();
        self.client.storage.create_run(&self.id, start_time)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}