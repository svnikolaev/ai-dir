use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "go",
        vec![
            ("func", Regex::new(r"(?m)^\s*func\s+(\w+)\s*\(").unwrap()),
            (
                "type",
                Regex::new(r"(?m)^\s*type\s+(\w+)\s+(?:struct|interface)").unwrap(),
            ),
            ("const", Regex::new(r"(?m)^\s*const\s+(\w+)").unwrap()),
        ],
    );
}
