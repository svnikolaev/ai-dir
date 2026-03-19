use crate::core::cache::{Cache, FileMetadata};
use crate::core::config::Config;
use crate::core::types::{Description, FileEntry};
use std::fs;

mod llm;
mod pattern;

use pattern::LANGUAGE_PATTERNS;
use pattern::rust::extract_from_rust; // добавляем use

const PATTERN_CACHE_VERSION: u32 = 1;
const DEFAULT_LONG_THRESHOLD: usize = 20;

fn count_lines(content: &str) -> usize {
    content.lines().count()
}

/// Общая функция для извлечения функций в зависимости от языка
fn extract_functions(ext: &str, content: &str) -> Vec<(String, usize)> {
    match ext {
        "rs" => {
            let (_, functions) = extract_from_rust(content);
            functions
        }
        _ => Vec::new(),
    }
}

pub fn describe_files_pattern(
    files: &[FileEntry],
    cache: &mut Cache,
    no_truncate: bool,
    config: &Config,
) -> Vec<Description> {
    let mut results = Vec::new();
    let params = serde_json::json!({
        "include": config.include_pattern,
        "exclude": config.exclude_pattern,
        "version": PATTERN_CACHE_VERSION,
    });

    for file in files {
        if let Some(max_size) = config.max_file_size {
            if let Ok(meta) = fs::metadata(&file.path) {
                if meta.len() > max_size {
                    results.push(Description::error(
                        file,
                        format!("[SKIPPED: file too large ({} > {})]", meta.len(), max_size),
                    ));
                    continue;
                }
            }
        }

        if let Some((cached_text, metadata)) = cache.get(&file.path, "pattern", &params) {
            let (functions, symbols) = if let Some(meta) = metadata {
                let functions = if meta.functions.is_empty() && !meta.long_functions.is_empty() {
                    meta.long_functions.clone()
                } else {
                    meta.functions.clone()
                };
                (functions, meta.symbols.clone())
            } else {
                (Vec::new(), Vec::new())
            };
            let long_functions: Vec<_> = functions
                .iter()
                .filter(|(_, lines)| *lines >= DEFAULT_LONG_THRESHOLD)
                .cloned()
                .collect();
            results.push(Description::cached(
                file,
                cached_text.to_string(),
                metadata.map(|m| m.total_lines),
                long_functions,
                functions,
                symbols,
            ));
            continue;
        }

        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(_) => {
                results.push(Description::error(file, "unreadable".into()));
                continue;
            }
        };

        let total_lines = count_lines(&content);
        let mut unique_symbols = Vec::new();
        let mut functions = Vec::new();

        if let Some(ext) = file.relative.split('.').last() {
            if ext == "rs" {
                let (symbols, funcs) = extract_from_rust(&content);
                unique_symbols = symbols;
                functions = funcs;
            } else if let Some(patterns) = LANGUAGE_PATTERNS.get(ext) {
                let mut raw_symbols = Vec::new();
                for (type_name, re) in patterns {
                    for cap in re.captures_iter(&content) {
                        let sym = format!("{} {}", type_name, &cap[1]);
                        raw_symbols.push(sym.clone());
                        if !unique_symbols.contains(&sym) {
                            unique_symbols.push(sym);
                        }
                    }
                }
                functions = extract_functions(ext, &content);
            }
        }

        let text = if unique_symbols.is_empty() {
            "no symbols".into()
        } else {
            let mut display = Vec::new();
            if no_truncate {
                display = unique_symbols.clone();
            } else {
                for s in &unique_symbols {
                    if display.len() < 10 {
                        display.push(s.clone());
                    } else {
                        break;
                    }
                }
                if unique_symbols.len() > 10 {
                    display.push("…".to_string());
                }
            }
            display.join(", ")
        };

        let long_functions: Vec<_> = functions
            .iter()
            .filter(|(_, lines)| *lines >= DEFAULT_LONG_THRESHOLD)
            .cloned()
            .collect();

        let metadata = FileMetadata {
            total_lines,
            long_functions: long_functions.clone(),
            functions: functions.clone(),
            symbols: unique_symbols.clone(),
        };
        cache.insert(
            file.path.clone(),
            text.clone(),
            "pattern".to_string(),
            params.clone(),
            Some(metadata),
        );

        results.push(Description {
            path: file.path.clone(),
            relative: file.relative.clone(),
            text,
            error: None,
            from_cache: false,
            total_lines: Some(total_lines),
            long_functions,
            functions,
            symbols: unique_symbols,
        });
    }

    results
}

pub use llm::describe_files as describe_files_llm;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_describe_files_pattern_skips_large_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.rs");
        let mut file = File::create(&file_path).unwrap();
        let data = vec![b'a'; 2 * 1024 * 1024];
        file.write_all(&data).unwrap();
        drop(file);

        let entry = FileEntry {
            path: file_path.clone(),
            relative: "large.rs".into(),
        };

        let mut cache = Cache::new();
        let mut config = Config::default();
        config.max_file_size = Some(1024 * 1024);

        let results = describe_files_pattern(&[entry], &mut cache, false, &config);
        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_some());
        assert!(results[0].error.as_ref().unwrap().contains("SKIPPED"));
    }

    #[test]
    fn test_rust_integration() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let content = r#"
            struct MyStruct;
            fn func1() {}
            fn func2() {
                // много строк
            }
        "#;
        std::fs::write(&file_path, content).unwrap();
        let entry = FileEntry {
            path: file_path,
            relative: "test.rs".into(),
        };
        let mut cache = Cache::new();
        let config = Config::default();
        let results = describe_files_pattern(&[entry], &mut cache, true, &config);
        assert_eq!(results.len(), 1);
        let desc = &results[0];
        assert!(desc.symbols.contains(&"struct MyStruct".to_string()));
        assert!(desc.symbols.contains(&"fn func1".to_string()));
        assert!(desc.symbols.contains(&"fn func2".to_string()));
        assert_eq!(desc.functions.len(), 2);
    }
}
