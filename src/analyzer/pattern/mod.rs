use std::collections::HashMap;
use std::fs;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::cache::Cache;
use crate::types::{Description, FileEntry};

// Объявляем подмодули
mod rust;
mod python;
mod javascript;
mod go;
mod java;
mod cfamily;
mod ruby;
mod swift;
mod kotlin;
mod markup;

// Тип для хранения паттернов: вектор пар (имя_типа, Regex)
type LanguagePatterns = Vec<(&'static str, Regex)>;

// Статическая карта расширение -> паттерны
static LANGUAGE_PATTERNS: Lazy<HashMap<&'static str, LanguagePatterns>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Регистрируем паттерны из каждого модуля
    rust::patterns(&mut m);
    python::patterns(&mut m);
    javascript::patterns(&mut m);
    go::patterns(&mut m);
    java::patterns(&mut m);
    cfamily::patterns(&mut m);
    ruby::patterns(&mut m);
    swift::patterns(&mut m);
    kotlin::patterns(&mut m);
    markup::patterns(&mut m);

    m
});

pub fn describe_files(files: &[FileEntry], cache: &mut Cache) -> Vec<Description> {
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