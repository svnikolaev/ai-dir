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

#[cfg(test)]
mod tests {
    use crate::commands::analyze::extract_functions_rust;

    #[test]
    fn test_extract_functions_rust_complex() {
        let code = r#"
            fn outer() {
                let s = "}";
                fn inner() {
                    // { comment
                }
                if true { 
                    let x = 1; 
                }
            }
            fn simple() {}
        "#;
        let functions = extract_functions_rust(code);
        assert_eq!(functions.len(), 3);
        let outer = functions.iter().find(|(name, _)| name == "outer").unwrap();
        assert!(outer.1 > 5);
        let inner = functions.iter().find(|(name, _)| name == "inner").unwrap();
        assert_eq!(inner.1, 3);
        let simple = functions.iter().find(|(name, _)| name == "simple").unwrap();
        assert_eq!(simple.1, 1);
    }
}
