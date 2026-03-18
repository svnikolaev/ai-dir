use super::LanguagePatterns;
use regex::Regex;
use std::collections::HashMap;

pub fn patterns(map: &mut HashMap<&'static str, LanguagePatterns>) {
    map.insert(
        "rs",
        vec![
            (
                "struct",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*(?:pub\s+)?struct\s+(\w+)").unwrap(),
            ),
            (
                "enum",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*(?:pub\s+)?enum\s+(\w+)").unwrap(),
            ),
            (
                "fn",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(")
                    .unwrap(),
            ),
            (
                "impl",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*impl\s+(\w+)").unwrap(),
            ),
            (
                "trait",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*(?:pub\s+)?trait\s+(\w+)").unwrap(),
            ),
            (
                "type",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*(?:pub\s+)?type\s+(\w+)").unwrap(),
            ),
            (
                "const",
                Regex::new(r"(?m)^\s*(?:#\[.*\]\s*)*(?:pub\s+)?const\s+(\w+)\s*:").unwrap(),
            ),
        ],
    );
}
