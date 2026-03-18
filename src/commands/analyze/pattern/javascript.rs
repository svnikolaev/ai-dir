use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    for ext in &["js", "jsx", "ts", "tsx"] {
        map.insert(
            ext,
            vec![
                (
                    "function",
                    Regex::new(r"(?m)^\s*function\s+(\w+)\s*\(").unwrap(),
                ),
                ("class", Regex::new(r"(?m)^\s*class\s+(\w+)").unwrap()),
                ("const", Regex::new(r"(?m)^\s*const\s+(\w+)\s*=").unwrap()),
                ("let", Regex::new(r"(?m)^\s*let\s+(\w+)\s*=").unwrap()),
            ],
        );
    }
}
