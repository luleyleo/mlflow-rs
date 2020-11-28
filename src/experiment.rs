use crate::{
    storage::ExperimentStorage,
    storage::{errors, primitive},
    Run,
};

/// A MLflow Experiment.
///
/// This can be created using [`Client::create_experiment`].
/// It can be used to group and create [`Run`]s.
pub struct Experiment {
    storage: Box<dyn ExperimentStorage>,
    id: String,
    name: String,
}

impl Experiment {
    pub(crate) fn new(
        storage: Box<dyn ExperimentStorage>,
        experiment: primitive::Experiment,
    ) -> Self {
        Experiment {
            storage,
            id: experiment.experiment_id,
            name: experiment.name,
        }
    }
}

/// Experiment methods without error handling.
impl Experiment {
    pub fn create_run(&mut self) -> Run {
        self.try_create_run().unwrap()
    }
}

/// Experiment methods with error handling.
impl Experiment {
    pub fn try_create_run(&mut self) -> Result<Run, errors::StorageError> {
        let start_time = crate::timestamp();
        self.storage.create_run(&self.id, start_time)
    }
}

/// Experiment information getters.
impl Experiment {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
