use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "swift",
        vec![
            (
                "class",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*class\s+(\w+)").unwrap(),
            ),
            (
                "struct",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*struct\s+(\w+)").unwrap(),
            ),
            (
                "enum",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*enum\s+(\w+)").unwrap(),
            ),
            (
                "protocol",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*protocol\s+(\w+)").unwrap(),
            ),
            (
                "func",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*func\s+(\w+)\s*\(").unwrap(),
            ),
        ],
    );
}
