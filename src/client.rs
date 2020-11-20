use crate::{
    storage::{
        self,
        errors::CreateExperimentError,
        errors::{GetExperimentError, StorageError},
        Storage,
    },
    Experiment,
};
use std::sync::Arc;

pub struct Client {
    pub(crate) storage: Arc<dyn Storage>,
}

impl Client {
    pub fn for_server(url: &str) -> Self {
        Client {
            storage: Arc::new(storage::Server::new(url)),
        }
    }

    pub fn create_experiment(&self, name: &str) -> Option<Experiment> {
        match self.try_create_experiment(name) {
            Ok(experiment) => Some(experiment),
            Err(CreateExperimentError::AlreadyExists(_)) => None,
            Err(err @ CreateExperimentError::Storage(_)) => {
                panic!("{}", err);
            }
        }
    }
    pub fn try_create_experiment(&self, name: &str) -> Result<Experiment, CreateExperimentError> {
        let primitive = self.storage.create_experiment(name)?;
        Ok(Experiment::new(self, primitive))
    }

    pub fn get_experiment(&self, name: &str) -> Option<Experiment> {
        match self.try_get_experiment(name) {
            Ok(experiment) => Some(experiment),
            Err(GetExperimentError::DoesNotExist(_)) => None,
            Err(err @ GetExperimentError::Storage(_)) => {
                panic!("{}", err);
            }
        }
    }
    pub fn try_get_experiment(&self, name: &str) -> Result<Experiment, GetExperimentError> {
        let primitive = self.storage.get_experiment(name)?;
        Ok(Experiment::new(self, primitive))
    }

    pub fn list_experiments(&self) -> Vec<Experiment> {
        self.try_list_experiments().unwrap()
    }
    pub fn try_list_experiments(&self) -> Result<Vec<Experiment>, StorageError> {
        let primitives = self.storage.list_experiments()?;
        Ok(primitives
            .into_iter()
            .map(|e| Experiment::new(self, e))
            .collect())
    }
}
