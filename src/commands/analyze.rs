use crate::core::cache::{Cache, FileMetadata};
use crate::core::types::{Description, FileEntry};
use regex::Regex;
use std::fs;

pub mod llm;
pub mod pattern;

use pattern::LANGUAGE_PATTERNS;

const LONG_FUNCTION_THRESHOLD: usize = 20;

fn count_lines(content: &str) -> usize {
    content.lines().count()
}

fn find_long_functions_rust(content: &str) -> Vec<(String, usize)> {
    let mut result = Vec::new();
    let fn_re = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(").unwrap();
    let lines: Vec<&str> = content.lines().collect();
    for cap in fn_re.captures_iter(content) {
        let fn_name = cap[1].to_string();
        let start_pos = cap.get(0).unwrap().start();
        let start_line = content[..start_pos].lines().count();
        let mut brace_level = 0;
        let mut end_line = start_line;
        let mut found = false;
        for (i, line) in lines.iter().enumerate().skip(start_line) {
            for ch in line.chars() {
                match ch {
                    '{' => brace_level += 1,
                    '}' => {
                        if brace_level > 0 {
                            brace_level -= 1;
                            if brace_level == 0 {
                                end_line = i;
                                found = true;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if found {
                break;
            }
        }
        if found {
            let fn_lines = end_line - start_line + 1;
            if fn_lines >= LONG_FUNCTION_THRESHOLD {
                result.push((fn_name, fn_lines));
            }
        }
    }
    result
}

fn find_long_functions(ext: &str, content: &str) -> Vec<(String, usize)> {
    match ext {
        "rs" => find_long_functions_rust(content),
        _ => Vec::new(),
    }
}

pub fn describe_files_pattern(
    files: &[FileEntry],
    cache: &mut Cache,
    no_truncate: bool,
) -> Vec<Description> {
    let mut results = Vec::new();
    let params = serde_json::json!({});

    for file in files {
        if let Some((cached_text, metadata)) = cache.get(&file.path, "pattern", &params) {
            results.push(Description::cached(
                file,
                cached_text.to_string(),
                metadata.map(|m| m.total_lines),
                metadata
                    .map(|m| m.long_functions.clone())
                    .unwrap_or_default(),
                metadata.map(|m| m.symbols.clone()).unwrap_or_default(),
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
        let mut raw_symbols = Vec::new(); // все найденные символы (могут повторяться)
        let mut unique_symbols = Vec::new(); // уникальные символы для отображения
        let mut long_functions = Vec::new();
        if let Some(ext) = file.relative.split('.').last() {
            if let Some(patterns) = LANGUAGE_PATTERNS.get(ext) {
                for (type_name, re) in patterns {
                    for cap in re.captures_iter(&content) {
                        let sym = format!("{} {}", type_name, &cap[1]);
                        raw_symbols.push(sym.clone());
                        if !unique_symbols.contains(&sym) {
                            unique_symbols.push(sym);
                        }
                    }
                }
            }
            long_functions = find_long_functions(ext, &content);
        }

        // Формируем текст для отображения (влияет only на human-readable форматы)
        let text = if raw_symbols.is_empty() {
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

        let metadata = FileMetadata {
            total_lines,
            long_functions: long_functions.clone(),
            symbols: unique_symbols.clone(), // сохраняем уникальные символы
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
            symbols: unique_symbols,
        });
    }

    results
}

pub use llm::describe_files as describe_files_llm;
