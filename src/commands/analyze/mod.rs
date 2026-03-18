use crate::core::cache::Cache;
use crate::core::types::{Description, FileEntry};
use std::fs;

// Подмодули
pub mod llm;
pub mod pattern;

// Импортируем готовую карту паттернов из модуля pattern
use pattern::LANGUAGE_PATTERNS;

/// Режим pattern: быстрое извлечение символов из файлов
pub fn describe_files_pattern(files: &[FileEntry], cache: &mut Cache) -> Vec<Description> {
    let mut results = Vec::new();

    for file in files {
        if let Some(cached) = cache.get(&file.path) {
            results.push(Description::cached(file, cached.to_string()));
            continue;
        }

        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(_) => {
                results.push(Description::error(file, "unreadable".into()));
                continue;
            }
        };

        let mut symbols = Vec::new();
        if let Some(ext) = file.relative.split('.').last() {
            if let Some(patterns) = LANGUAGE_PATTERNS.get(ext) {
                for (type_name, re) in patterns {
                    for cap in re.captures_iter(&content) {
                        symbols.push(format!("{} {}", type_name, &cap[1]));
                    }
                }
            }
        }

        let text = if symbols.is_empty() {
            "no symbols".into()
        } else {
            let mut unique = Vec::new();
            for s in &symbols {
                if !unique.contains(s) && unique.len() < 10 {
                    unique.push(s.clone());
                }
            }
            if symbols.len() > 10 {
                unique.push("…".to_string());
            }
            unique.join(", ")
        };

        cache.insert(file.path.clone(), text.clone());
        results.push(Description::new(file, text));
    }

    results
}

// Реэкспорт функции из llm для удобства
pub use llm::describe_files as describe_files_llm;
