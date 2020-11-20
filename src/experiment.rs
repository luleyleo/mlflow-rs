use crate::{Client, Id, Run, storage::errors::StorageError};
use crate::storage::primitive;

pub struct Experiment<'a> {
    pub(crate) client: &'a Client,
    id: Id,
    name: String,
}

impl<'a> Experiment<'a> {
    pub(crate) fn new(client: &'a Client, experiment: primitive::Experiment) -> Self {
        Experiment {
            client,
            id: experiment.experiment_id,
            name: experiment.name,
        }
    }

    pub fn create_run(&self) -> Run {
        self.try_create_run().unwrap()
    }

    pub fn try_create_run(&self) -> Result<Run, StorageError> {
        let start_time = crate::time_stamp();
        let primitive = self.client.storage.create_run(&self.id, start_time)?;
        Ok(Run::new(self, primitive))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}