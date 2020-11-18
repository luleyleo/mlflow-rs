use std::sync::Arc;
use crate::{Experiment, storage::{self, Storage}};

pub struct Client {
    pub(crate) storage: Arc<dyn Storage>,
}

impl Client {
    pub fn for_server(url: &str) -> Self {
        Client {
            storage: Arc::new(storage::Server::new(url))
        }
    }

    pub fn create_experiment(&self, name: &str) -> Experiment {
        self.storage.create_experiment(name)
    }

    pub fn get_experiment(&self, name: &str) -> Option<Experiment> {
        self.storage.get_experiment(name)
    }

    pub fn list_experiments(&self) -> Vec<Experiment> {
        self.storage.list_experiments()
    }
}