use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
            Ok(time) => time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64,
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