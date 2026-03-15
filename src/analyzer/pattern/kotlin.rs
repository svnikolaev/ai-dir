use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "kt",
        vec![
            (
                "class",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*class\s+(\w+)").unwrap(),
            ),
            (
                "interface",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*interface\s+(\w+)").unwrap(),
            ),
            (
                "enum",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*enum\s+(\w+)").unwrap(),
            ),
            (
                "object",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*object\s+(\w+)").unwrap(),
            ),
            (
                "fun",
                Regex::new(r"(?m)^\s*(?:public|private|internal)?\s*fun\s+(\w+)\s*\(").unwrap(),
            ),
        ],
    );
}
