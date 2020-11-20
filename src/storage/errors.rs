use thiserror::Error;

pub type StorageError = anyhow::Error;

#[derive(Error, Debug)]
pub enum CreateExperimentError {
    #[error("the experiment {0} already exists")]
    AlreadyExists(String),
    #[error("an error ocurred in the storage backend")]
    Storage(#[from] StorageError),
}

#[derive(Error, Debug)]
pub enum GetExperimentError {
    #[error("the experiment {0} does not exist")]
    DoesNotExist(String),
    #[error("an error ocurred in the storage backend")]
    Storage(#[from] StorageError),
}
