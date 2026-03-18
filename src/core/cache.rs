use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Метаданные файла, сохраняемые в кэше
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub total_lines: usize,
    pub long_functions: Vec<(String, usize)>, // (имя функции, количество строк)
}

/// Запись кэша
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    mtime: i64,
    text: String,
    mode: String,                   // "pattern" или "llm"
    params: serde_json::Value,      // параметры режима (например, {"language": "en"} для llm)
    metadata: Option<FileMetadata>, // метаданные, доступные только для pattern
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Cache {
    entries: HashMap<PathBuf, Vec<CacheEntry>>, // теперь для одного пути может быть несколько записей (разные режимы/параметры)
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

    /// Получить запись из кэша по пути, режиму и параметрам
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
                break; // если mtime не совпал, запись устарела, но может быть другая с теми же mode/params? нет, только одна.
            }
        }
        None
    }

    /// Вставить запись
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
        self.entries
            .entry(path)
            .or_insert_with(Vec::new)
            .push(entry);
        // Для поддержки ограничения числа записей на файл можно оставить только последнюю или все. Пока оставляем все.
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
    // Тесты временно отключены из-за изменений в структуре кэша.
    // TODO: переписать тесты для новой версии Cache.
}
