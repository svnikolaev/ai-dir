// src/core/cache.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub total_lines: usize,
    pub long_functions: Vec<(String, usize)>, // устарело, для обратной совместимости
    pub functions: Vec<(String, usize)>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    mtime: i64,
    text: String,
    mode: String,
    params: serde_json::Value,
    metadata: Option<FileMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cache {
    entries: HashMap<PathBuf, Vec<CacheEntry>>,
    #[serde(skip)]
    path: PathBuf,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            path: default_cache_path(),
        }
    }

    pub fn load() -> Result<Self> {
        let path = default_cache_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut cache: Cache = serde_json::from_str(&content)?;
            cache.path = path;
            Ok(cache)
        } else {
            Ok(Self {
                entries: HashMap::new(),
                path,
            })
        }
    }

    #[allow(dead_code)]
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let mut cache: Cache = serde_json::from_str(&content)?;
            cache.path = path.to_path_buf();
            Ok(cache)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Атомарное сохранение: пишем во временный файл, затем переименовываем
        let temp_path = self.path.with_extension("tmp");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, &self.path)?;
        Ok(())
    }

    pub fn get(
        &self,
        path: &Path,
        mode: &str,
        params: &serde_json::Value,
    ) -> Option<(&str, Option<&FileMetadata>)> {
        let entries = self.entries.get(path)?;
        for entry in entries {
            if entry.mode == mode && &entry.params == params {
                if let Ok(meta) = fs::metadata(path) {
                    if let Ok(mtime) = meta.modified() {
                        let mtime_ms = mtime
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as i64;
                        if mtime_ms == entry.mtime {
                            return Some((&entry.text, entry.metadata.as_ref()));
                        }
                    }
                }
                break;
            }
        }
        None
    }

    pub fn insert(
        &mut self,
        path: PathBuf,
        text: String,
        mode: String,
        params: serde_json::Value,
        metadata: Option<FileMetadata>,
    ) {
        let mtime = match fs::metadata(&path).and_then(|m| m.modified()) {
            Ok(time) => time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            Err(_) => return,
        };
        let entry = CacheEntry {
            mtime,
            text,
            mode,
            params,
            metadata,
        };
        // Заменяем все старые записи для этого пути новой (оставляем только последнюю)
        self.entries.insert(path, vec![entry]);
    }
}

fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai-dir")
        .join("cache.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_cache_insert_replaces_old_entries() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.rs");
        File::create(&file_path).unwrap(); // создаём файл

        let mut cache = Cache::new();
        let path = file_path.clone();

        cache.insert(
            path.clone(),
            "text1".into(),
            "mode".into(),
            json!({"p": 1}),
            None,
        );
        cache.insert(
            path.clone(),
            "text2".into(),
            "mode".into(),
            json!({"p": 2}),
            None,
        );

        let entries = cache.entries.get(&path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "text2");
    }

    #[test]
    fn test_cache_get_respects_params() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.rs");
        File::create(&file_path).unwrap(); // создаём файл

        let mut cache = Cache::new();
        let path = file_path;
        let params1 = json!({"hash": "abc"});
        let params2 = json!({"hash": "def"});

        cache.insert(
            path.clone(),
            "text1".into(),
            "mode".into(),
            params1.clone(),
            None,
        );
        // Вставляем вторую запись, она заменит первую
        cache.insert(
            path.clone(),
            "text2".into(),
            "mode".into(),
            params2.clone(),
            None,
        );

        // После замены первой записи второй, первая не должна быть доступна
        assert!(cache.get(&path, "mode", &params1).is_none());
        // Вторая должна быть доступна
        assert!(cache.get(&path, "mode", &params2).is_some());
    }
}
