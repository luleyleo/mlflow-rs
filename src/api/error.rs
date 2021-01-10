use thiserror::Error;

pub type StorageError = anyhow::Error;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("the resource {0} already exists")]
    AlreadyExists(String),
    #[error("an error ocurred in the storage backend: {0:?}")]
    Storage(#[from] StorageError),
}

#[derive(Error, Debug)]
pub enum GetError {
    #[error("the resource {0} does not exist")]
    DoesNotExist(String),
    #[error("an error ocurred in the storage backend: {0:?}")]
    Storage(#[from] StorageError),
}

#[derive(Error, Debug)]
pub enum BatchError {
    #[error("only up to 1000 items can be logged at once, found {0}")]
    ToManyItems(u32),
    #[error("only up to 1000 metrics can be logged at once, found {0}")]
    ToManyMetrics(u32),
    #[error("only up to 100 params can be logged at once, found {0}")]
    ToManyParams(u32),
    #[error("only up to 100 tags can be logged at once, found {0}")]
    ToManyTags(u32),
    #[error("an error ocurred in the storage backend: {0:?}")]
    Storage(#[from] StorageError),
}

pub type DeleteError = GetError;
pub type UpdateError = GetError;
