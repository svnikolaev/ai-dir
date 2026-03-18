use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "py",
        vec![
            ("class", Regex::new(r"(?m)^\s*class\s+(\w+)").unwrap()),
            ("def", Regex::new(r"(?m)^\s*def\s+(\w+)\s*\(").unwrap()),
            (
                "async def",
                Regex::new(r"(?m)^\s*async\s+def\s+(\w+)\s*\(").unwrap(),
            ),
            (
                "const",
                Regex::new(r"(?m)^\s*([A-Z][A-Z0-9_]+)\s*=").unwrap(),
            ),
        ],
    );
}
