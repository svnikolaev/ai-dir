use crate::config::Config;
use crate::types::FileEntry;
use anyhow::Result;
use ignore::WalkBuilder;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn scan(root: &Path, config: &Config) -> Result<Vec<FileEntry>> {
    // Приводим корневой путь к абсолютному для корректного strip_prefix
    let root = fs::canonicalize(root)?;

    let include_re = Regex::new(&config.include_pattern)?;
    let exclude_re = Regex::new(&config.exclude_pattern)?;
    let mut files = Vec::new();

    for result in WalkBuilder::new(&root)
        .git_ignore(true)
        .ignore(false)
        .hidden(false)
        .build()
    {
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
        if exclude_re.is_match(&relative) {
            continue;
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
