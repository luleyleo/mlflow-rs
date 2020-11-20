use crate::{Client, Run, storage::{errors, primitive}};

/// A MLflow Experiment.
///
/// This can be created using [`Client::create_experiment`].
/// It can be used to group and create [`Run`]s.
pub struct Experiment<'a> {
    pub(crate) client: &'a Client,
    id: String,
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
}

/// Experiment methods without error handling.
impl Experiment<'_> {
    pub fn create_run(&self) -> Run {
        self.try_create_run().unwrap()
    }
}

/// Experiment methods with error handling.
impl Experiment<'_> {
    pub fn try_create_run(&self) -> Result<Run, errors::StorageError> {
        let start_time = crate::timestamp();
        let primitive = self.client.storage.create_run(&self.id, start_time)?;
        Ok(Run::new(self, primitive))
    }
}

/// Experiment information getters.
impl Experiment<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
