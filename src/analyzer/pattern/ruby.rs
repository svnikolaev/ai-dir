use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "rb",
        vec![
            ("class", Regex::new(r"(?m)^\s*class\s+(\w+)").unwrap()),
            ("module", Regex::new(r"(?m)^\s*module\s+(\w+)").unwrap()),
            ("def", Regex::new(r"(?m)^\s*def\s+(\w+)\s*\(").unwrap()),
        ],
    );
}
