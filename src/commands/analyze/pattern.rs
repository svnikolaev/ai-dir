use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

// Подключаем языковые модули
pub mod cfamily;
pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod markup;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;

// Тип для хранения паттернов: вектор пар (имя_типа, Regex)
pub type LanguagePatterns = Vec<(&'static str, Regex)>;

// Статическая карта расширение -> паттерны
pub static LANGUAGE_PATTERNS: Lazy<HashMap<&'static str, LanguagePatterns>> = Lazy::new(|| {
    let mut m = HashMap::new();

    rust::patterns(&mut m);
    python::patterns(&mut m);
    javascript::patterns(&mut m);
    go::patterns(&mut m);
    java::patterns(&mut m);
    cfamily::patterns(&mut m);
    ruby::patterns(&mut m);
    swift::patterns(&mut m);
    kotlin::patterns(&mut m);
    markup::patterns(&mut m);

    m
});

#[cfg(test)]
mod tests {
    use crate::core::cache::Cache;
    use crate::core::config::Config;
    use crate::core::types::FileEntry;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_file(content: &str, ext: &str) -> (FileEntry, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join(format!("test.{}", ext));
        let mut f = File::create(&path).unwrap();
        write!(f, "{}", content).unwrap();
        let entry = FileEntry {
            path: path.clone(),
            relative: format!("test.{}", ext),
        };
        (entry, dir)
    }

    #[test]
    fn test_rust_patterns() {
        let content = r#"
            struct MyStruct;
            enum MyEnum { A, B }
            fn my_function() {}
            pub fn public_fn() {}
            async fn async_fn() {}
            impl MyStruct {}
            trait MyTrait {}
            type MyType = i32;
            const MY_CONST: i32 = 42;
        "#;
        let (entry, _dir) = create_file(content, "rs");
        let mut cache = Cache::new();
        let config = Config::default();
        let result = super::super::describe_files_pattern(&[entry], &mut cache, false, &config);
        assert_eq!(result.len(), 1);
        let desc = &result[0];
        assert!(desc.text.contains("struct MyStruct"));
        assert!(desc.text.contains("enum MyEnum"));
        assert!(desc.text.contains("fn my_function"));
        assert!(desc.text.contains("fn public_fn"));
        assert!(desc.text.contains("fn async_fn"));
        assert!(desc.text.contains("impl MyStruct"));
        assert!(desc.text.contains("trait MyTrait"));
        assert!(desc.text.contains("type MyType"));
        assert!(desc.text.contains("const MY_CONST"));
        assert!(!desc.text.contains("…"));
        // Проверяем functions
        assert_eq!(desc.functions.len(), 3); // my_function, public_fn, async_fn
        assert_eq!(desc.symbols.len(), 9);
    }

    #[test]
    fn test_no_symbols() {
        let content = "just plain text without any symbols";
        let (entry, _dir) = create_file(content, "txt");
        let mut cache = Cache::new();
        let config = Config::default();
        let result = super::super::describe_files_pattern(&[entry], &mut cache, false, &config);
        assert_eq!(result[0].text, "no symbols");
        assert!(result[0].symbols.is_empty());
        assert!(result[0].functions.is_empty());
    }
}
