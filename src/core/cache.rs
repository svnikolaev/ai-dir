use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    mtime: i64,
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cache {
    entries: HashMap<PathBuf, CacheEntry>,
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

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn get(&self, path: &Path) -> Option<&str> {
        let entry = self.entries.get(path)?;
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(mtime) = meta.modified() {
                let mtime_ms = mtime
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;
                if mtime_ms == entry.mtime {
                    return Some(&entry.text);
                }
            }
        }
        None
    }

    pub fn insert(&mut self, path: PathBuf, text: String) {
        let mtime = match fs::metadata(&path).and_then(|m| m.modified()) {
            Ok(time) => time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            Err(_) => return,
        };
        self.entries.insert(path, CacheEntry { mtime, text });
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_cache_insert_and_get() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path)
            .unwrap()
            .write_all(b"content")
            .unwrap();

        let mut cache = Cache::new();
        cache.insert(file_path.clone(), "description".into());

        // Проверим, что get возвращает значение, т.к. mtime совпадает
        assert_eq!(cache.get(&file_path), Some("description"));
    }

    #[test]
    fn test_cache_invalidation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"content").unwrap();
        let _mtime1 = file.metadata().unwrap().modified().unwrap();

        let mut cache = Cache::new();
        cache.insert(file_path.clone(), "desc".into());

        // изменим файл
        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"new content").unwrap();
        file.sync_all().unwrap();

        // теперь get должен вернуть None
        assert_eq!(cache.get(&file_path), None);
    }

    #[test]
    fn test_cache_save_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path)
            .unwrap()
            .write_all(b"content")
            .unwrap();

        let cache_path = dir.path().join("cache.json");
        let mut cache = Cache::new();
        cache.path = cache_path.clone();
        cache.insert(file_path.clone(), "saved".into());
        cache.save().unwrap();

        // Загружаем и проверяем содержимое
        let loaded: Cache =
            serde_json::from_str(&std::fs::read_to_string(&cache_path).unwrap()).unwrap();
        assert!(loaded.entries.contains_key(&file_path));
        assert_eq!(loaded.entries.get(&file_path).unwrap().text, "saved");
    }
}
