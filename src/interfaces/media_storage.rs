use std::{
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;

use crate::errors::app_error::AppError;

#[async_trait]
pub trait MediaStorage: Send + Sync {
    async fn write(&self, object_key: &str, data: &[u8]) -> Result<(), AppError>;
    async fn read(&self, object_key: &str) -> Result<Vec<u8>, AppError>;
    async fn remove(&self, object_key: &str) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct LocalFileMediaStorage {
    base_path: PathBuf,
}

impl LocalFileMediaStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    fn resolve(&self, object_key: &str) -> Result<PathBuf, AppError> {
        if object_key.trim().is_empty()
            || object_key.contains('\\')
            || object_key.contains("..")
            || object_key.starts_with('/')
            || Path::new(object_key)
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(AppError::Internal);
        }
        let candidate = self.base_path.join(object_key);
        if !candidate.starts_with(&self.base_path) {
            return Err(AppError::Internal);
        }
        Ok(candidate)
    }
}

#[async_trait]
impl MediaStorage for LocalFileMediaStorage {
    async fn write(&self, object_key: &str, data: &[u8]) -> Result<(), AppError> {
        let path = self.resolve(object_key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AppError::Internal)?;
        }
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, data).map_err(|_| AppError::Internal)?;
        fs::rename(&temp_path, &path).map_err(|_| AppError::Internal)
    }

    async fn read(&self, object_key: &str) -> Result<Vec<u8>, AppError> {
        let path = self.resolve(object_key)?;
        fs::read(&path).map_err(|_| AppError::Internal)
    }

    async fn remove(&self, object_key: &str) -> Result<(), AppError> {
        let path = self.resolve(object_key)?;
        fs::remove_file(&path).map_err(|_| AppError::Internal)
    }
}
