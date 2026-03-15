use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    // C / C++
    for ext in &["c", "h", "cpp", "hpp", "cc", "cxx"] {
        map.insert(
            ext,
            vec![
                ("struct", Regex::new(r"(?m)^\s*struct\s+(\w+)").unwrap()),
                ("class", Regex::new(r"(?m)^\s*class\s+(\w+)").unwrap()),
                ("enum", Regex::new(r"(?m)^\s*enum\s+(\w+)").unwrap()),
                (
                    "function",
                    Regex::new(r"(?m)^\s*\w+\s+(\w+)\s*\([^)]*\)\s*\{").unwrap(),
                ),
            ],
        );
    }

    // C#
    map.insert(
        "cs",
        vec![
            ("class", Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*class\s+(\w+)").unwrap()),
            ("interface", Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*interface\s+(\w+)").unwrap()),
            ("enum", Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*enum\s+(\w+)").unwrap()),
            ("struct", Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*struct\s+(\w+)").unwrap()),
            ("method", Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*(?:async\s+)?(?:\w+\s+)*(\w+)\s*\([^)]*\)\s*\{").unwrap()),
        ],
    );
}
