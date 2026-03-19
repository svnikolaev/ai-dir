use crate::core::cache::{Cache, FileMetadata};
use crate::core::config::Config;
use crate::core::types::{Description, FileEntry};
use std::fs;

mod llm;
mod pattern;

use pattern::LANGUAGE_PATTERNS;
use syn::spanned::Spanned;
use syn::{self, ImplItemFn, ItemFn, visit::Visit};

const PATTERN_CACHE_VERSION: u32 = 1;

fn count_lines(content: &str) -> usize {
    content.lines().count()
}

struct FunctionCollector {
    functions: Vec<(String, usize)>,
}

impl<'ast> Visit<'ast> for FunctionCollector {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        let end_line = node.span().end().line;
        let lines = end_line - start_line + 1;
        self.functions.push((name, lines));
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        let name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        let end_line = node.span().end().line;
        let lines = end_line - start_line + 1;
        self.functions.push((name, lines));
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Извлечение функций из Rust кода с точным определением строк с использованием `syn`.
fn extract_functions_rust(content: &str) -> Vec<(String, usize)> {
    let ast = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(_) => return Vec::new(),
    };
    let mut collector = FunctionCollector {
        functions: Vec::new(),
    };
    collector.visit_file(&ast);
    collector.functions
}

/// Общая функция для извлечения функций в зависимости от языка
fn extract_functions(ext: &str, content: &str) -> Vec<(String, usize)> {
    match ext {
        "rs" => extract_functions_rust(content),
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
            results.push(Description::cached(
                file,
                cached_text.to_string(),
                metadata.map(|m| m.total_lines),
                metadata
                    .map(|m| m.long_functions.clone())
                    .unwrap_or_default(),
                metadata.map(|m| m.functions.clone()).unwrap_or_default(),
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
        let mut raw_symbols = Vec::new();
        let mut unique_symbols = Vec::new();
        let mut functions = Vec::new();
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
            functions = extract_functions(ext, &content);
        }

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
            long_functions: vec![],
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
            long_functions: vec![],
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
        let data = vec![b'a'; 2 * 1024 * 1024]; // 2 МБ
        file.write_all(&data).unwrap();
        drop(file);

        let entry = FileEntry {
            path: file_path.clone(),
            relative: "large.rs".into(),
        };

        let mut cache = Cache::new();
        let mut config = Config::default();
        config.max_file_size = Some(1024 * 1024); // 1 МБ

        let results = describe_files_pattern(&[entry], &mut cache, false, &config);
        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_some());
        assert!(results[0].error.as_ref().unwrap().contains("SKIPPED"));
    }
}
