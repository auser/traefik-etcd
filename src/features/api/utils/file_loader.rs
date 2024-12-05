use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::error;
use walkdir::WalkDir;

use crate::config::traefik_config::TraefikConfigVersion;
use crate::features::{TraefikApiError, TraefikApiResult};
use crate::TraefikConfig;

// RAII-style lock guard
struct FileLock<'a> {
    path: PathBuf,
    locks: tokio::sync::MutexGuard<'a, HashMap<PathBuf, ()>>,
}

impl<'a> Drop for FileLock<'a> {
    fn drop(&mut self) {
        self.locks.remove(&self.path);
    }
}

async fn get_file_config(
    id: i64,
    file_config: &FileConfig,
) -> TraefikApiResult<TraefikConfigVersion> {
    let config_dir = &file_config.config_dir;

    // Find all YAML files in directory
    let mut yaml_files = vec![];
    if config_dir.exists() && config_dir.is_dir() {
        for entry in WalkDir::new(config_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "yaml" || ext == "yml")
            })
        {
            let meta = entry.metadata()?;
            yaml_files.push((entry.path().to_path_buf(), meta));
        }
    }

    // Sort files to ensure consistent ID mapping
    yaml_files.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Find file with corresponding negative ID
    // ID -2 corresponds to first file, -3 to second, etc.
    let file_index = (-id - 2) as usize;

    let (file_path, _metadata) = yaml_files.get(file_index).ok_or_else(|| {
        TraefikApiError::NotFound(format!("File-based configuration {} not found", id))
    })?;

    // Acquire lock for this file
    let _lock = file_config.acquire_lock(file_path).await;

    // Read and parse the file
    let content = fs::read_to_string(file_path).await?;

    // Validate YAML content
    serde_yaml::from_str::<TraefikConfig>(&content)?;

    let created_at = match fs::metadata(file_path).await?.created() {
        Ok(created_at) => created_at,
        Err(e) => {
            error!("error in get_yaml_configs: {:?}", e);
            Utc::now().into()
        }
    }
    .into();
    let updated_at = match fs::metadata(file_path).await?.modified() {
        Ok(updated_at) => updated_at,
        Err(e) => {
            error!("error in get_yaml_configs: {:?}", e);
            Utc::now().into()
        }
    }
    .into();

    Ok(TraefikConfigVersion {
        id,
        name: file_path
            .strip_prefix(config_dir)
            .unwrap_or(file_path)
            .display()
            .to_string(),
        config: serde_json::Value::String(content),
        created_at,
        updated_at,
        version: 1,
    })
}

// File cache implementation
pub struct FileCache {
    files: RwLock<HashMap<i64, (SystemTime, TraefikConfigVersion)>>,
    max_age: Duration,
}

impl FileCache {
    pub fn new(max_age: Duration) -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            max_age,
        }
    }

    pub async fn get_or_load(
        &self,
        id: i64,
        loader: impl std::future::Future<Output = TraefikApiResult<TraefikConfigVersion>>,
    ) -> TraefikApiResult<TraefikConfigVersion> {
        let now = SystemTime::now();

        // Check cache first
        if let Some((cached_time, config)) = self.files.read().await.get(&id) {
            if now.duration_since(*cached_time)? < self.max_age {
                return Ok(config.clone());
            }
        }

        // Load and cache if needed
        let config = loader.await?;
        self.files.write().await.insert(id, (now, config.clone()));

        Ok(config)
    }

    pub async fn invalidate(&self, id: i64) {
        self.files.write().await.remove(&id);
    }
}

#[derive(Clone)]
pub struct FileConfig {
    pub config_dir: PathBuf,
    file_locks: Arc<Mutex<HashMap<PathBuf, ()>>>,
    cache: Arc<FileCache>,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self::new("./frontend/templates")
    }
}

impl FileConfig {
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: templates_dir.into(),
            file_locks: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(FileCache::new(Duration::from_secs(60))), // 1 minute cache
        }
    }

    pub async fn acquire_lock(&self, path: &Path) -> impl Drop + '_ {
        let mut locks = self.file_locks.lock().await;
        locks.insert(path.to_path_buf(), ());
        FileLock {
            path: path.to_path_buf(),
            locks,
        }
    }

    pub async fn get_config_by_id(&self, id: i64) -> TraefikApiResult<TraefikConfigVersion> {
        self.cache.get_or_load(id, get_file_config(id, self)).await
    }
}

pub async fn save_file_config(
    id: i64,
    content: &str,
    file_config: &FileConfig,
) -> TraefikApiResult<TraefikConfigVersion> {
    let config_dir = &file_config.config_dir;

    // Validate YAML content before saving
    serde_yaml::from_str::<TraefikConfig>(content)?;

    // Find the file path same way as in get_file_config
    let mut yaml_files = vec![];
    if config_dir.exists() && config_dir.is_dir() {
        for entry in WalkDir::new(config_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "yaml" || ext == "yml")
            })
        {
            yaml_files.push(entry.path().to_path_buf());
        }
    }

    yaml_files.sort();
    let file_index = (-id - 2) as usize;
    let file_path = yaml_files.get(file_index).ok_or_else(|| {
        TraefikApiError::NotFound(format!("File-based configuration {} not found", id))
    })?;

    // Acquire lock for this file
    let _lock = file_config.acquire_lock(file_path).await;

    // Save the file
    fs::write(file_path, content).await?;

    // Invalidate cache for this ID
    file_config.cache.invalidate(id).await;

    // Get updated metadata
    let metadata = fs::metadata(file_path).await?;

    Ok(TraefikConfigVersion {
        id,
        name: file_path
            .strip_prefix(config_dir)
            .unwrap_or(file_path)
            .display()
            .to_string(),
        config: serde_json::Value::String(content.to_string()),
        created_at: DateTime::from(metadata.created()?),
        updated_at: DateTime::from(metadata.modified()?),
        version: 1,
    })
}
