use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    // TOML
    map.insert(
        "toml",
        vec![
            (
                "table",
                Regex::new(r"(?m)^\s*\[([a-zA-Z_][a-zA-Z0-9_.-]*)\]").unwrap(),
            ),
            (
                "key",
                Regex::new(r"(?m)^\s*([a-zA-Z_][a-zA-Z0-9_.-]*)\s*=").unwrap(),
            ),
        ],
    );

    // YAML
    for ext in &["yaml", "yml"] {
        map.insert(
            ext,
            vec![(
                "key",
                Regex::new(r"(?m)^\s*([a-zA-Z_][a-zA-Z0-9_-]*)\s*:").unwrap(),
            )],
        );
    }

    // JSON
    map.insert(
        "json",
        vec![("key", Regex::new(r#"(?m)^\s*"([^"]+)"\s*:"#).unwrap())],
    );

    // Markdown (заголовки)
    map.insert(
        "md",
        vec![("heading", Regex::new(r"(?m)^#+\s+(.+)").unwrap())],
    );
}
