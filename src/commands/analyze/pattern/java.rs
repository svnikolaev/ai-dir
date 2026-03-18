use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "java",
        vec![
            (
                "class",
                Regex::new(r"(?m)^\s*(?:public\s+)?class\s+(\w+)").unwrap(),
            ),
            (
                "interface",
                Regex::new(r"(?m)^\s*(?:public\s+)?interface\s+(\w+)").unwrap(),
            ),
            (
                "enum",
                Regex::new(r"(?m)^\s*(?:public\s+)?enum\s+(\w+)").unwrap(),
            ),
            (
                "method",
                Regex::new(
                    r"(?m)^\s*(?:public|private|protected)\s+(?:\w+\s+)*(\w+)\s*\([^)]*\)\s*\{",
                )
                .unwrap(),
            ),
        ],
    );
}
