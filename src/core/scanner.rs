use crate::core::config::Config;
use crate::core::types::FileEntry;
use anyhow::Result;
use ignore::WalkBuilder;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Сканирует директорию и возвращает список файлов, соответствующих настройкам.
pub fn scan(root: &Path, config: &Config) -> Result<Vec<FileEntry>> {
    // Приводим корневой путь к абсолютному для корректного strip_prefix
    let root = fs::canonicalize(root)?;

    let include_re = Regex::new(&config.include_pattern)?;
    // Создаём exclude_re только если паттерн не пустой
    let exclude_re = if config.exclude_pattern.is_empty() {
        None
    } else {
        Some(Regex::new(&config.exclude_pattern)?)
    };
    let mut files = Vec::new();

    // Настраиваем WalkBuilder с учётом .gitignore и скрытых файлов
    let mut walk_builder = WalkBuilder::new(&root);
    walk_builder
        .git_ignore(config.respect_gitignore) // уважать .gitignore если true
        .ignore(false) // не уважать .ignore файлы
        .hidden(false); // не игнорировать скрытые файлы

    for result in walk_builder.build() {
        let entry = result?;

        // Проверяем, что это файл (file_type может быть None для ссылок и т.п.)
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let absolute = entry.path().to_path_buf();
        let relative = absolute
            .strip_prefix(&root)
            .expect("entry should be inside root") // безопасно, т.к. WalkBuilder гарантирует
            .to_string_lossy()
            .replace('\\', "/");

        // Применяем пользовательские паттерны
        if let Some(exclude_re) = &exclude_re {
            if exclude_re.is_match(&relative) {
                continue;
            }
        }
        if !include_re.is_match(&relative) {
            continue;
        }

        files.push(FileEntry {
            path: absolute,
            relative,
        });
    }

    files.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use crate::core::types::Language;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::Builder;

    fn create_test_files(dir: &std::path::Path, files: &[&str]) {
        for f in files {
            let path = dir.join(f);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut file = File::create(path).unwrap();
            writeln!(file, "content").unwrap();
            file.sync_all().unwrap();
        }
    }

    #[test]
    fn test_scan_include_exclude() {
        let dir = Builder::new().prefix("ai_dir_test_").tempdir().unwrap();
        // Создаём пустой .gitignore, чтобы глобальные правила не влияли
        File::create(dir.path().join(".gitignore")).unwrap();
        create_test_files(
            dir.path(),
            &[
                "src/main.rs",
                "src/lib.rs",
                "tests/test.rs",
                "target/debug/app",
                "README.md",
                ".git/config",
            ],
        );

        let config = Config {
            include_pattern: r"\.(rs|md)$".to_string(),
            exclude_pattern: r"target|\.git".to_string(),
            default_mode: crate::core::config::Mode::Pattern,
            backends: vec![],
            cache_enabled: false,
            cache_ttl_days: None,
            language: Language::En,
            respect_gitignore: true,
            ..Default::default()
        };

        let files = scan(dir.path(), &config).unwrap();
        let paths: Vec<_> = files.iter().map(|f| f.relative.as_str()).collect();
        assert_eq!(
            paths,
            vec!["README.md", "src/lib.rs", "src/main.rs", "tests/test.rs"]
        );
    }

    #[test]
    fn test_scan_absolute_paths() {
        let dir = Builder::new().prefix("ai_dir_test_").tempdir().unwrap();
        File::create(dir.path().join(".gitignore")).unwrap();
        create_test_files(dir.path(), &["a.txt", "b.txt"]);

        let config = Config {
            include_pattern: r"\.txt$".to_string(),
            exclude_pattern: "".to_string(),
            default_mode: crate::core::config::Mode::Pattern,
            backends: vec![],
            cache_enabled: false,
            cache_ttl_days: None,
            language: Language::En,
            respect_gitignore: true,
            ..Default::default()
        };

        let files = scan(dir.path(), &config).unwrap();
        assert_eq!(files.len(), 2);
        for f in &files {
            assert!(f.path.is_absolute());
            assert!(f.relative == "a.txt" || f.relative == "b.txt");
        }
    }

    #[test]
    fn test_scan_sorting() {
        let dir = Builder::new().prefix("ai_dir_test_").tempdir().unwrap();
        File::create(dir.path().join(".gitignore")).unwrap();
        create_test_files(dir.path(), &["c.txt", "a/b.txt", "a.txt"]);

        let config = Config {
            include_pattern: r"\.txt$".to_string(),
            exclude_pattern: "".to_string(),
            default_mode: crate::core::config::Mode::Pattern,
            backends: vec![],
            cache_enabled: false,
            cache_ttl_days: None,
            language: Language::En,
            respect_gitignore: true,
            ..Default::default()
        };

        let files = scan(dir.path(), &config).unwrap();
        let paths: Vec<_> = files.iter().map(|f| f.relative.as_str()).collect();
        assert_eq!(paths, vec!["a.txt", "a/b.txt", "c.txt"]);
    }
}
