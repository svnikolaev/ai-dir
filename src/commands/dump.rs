use crate::core::types::FileEntry;
use crate::core::utils::{human_readable_size, is_binary, read_file_content, truncate_lines};
use anyhow::Result;
use std::io::{self};
use std::path::Path;
use std::time::UNIX_EPOCH;

mod markdown;
mod plain;
mod xml;

#[derive(Debug, Clone, Copy)]
pub enum DumpFormat {
    Plain,
    Markdown,
    Xml,
}

pub fn run(
    files: &[FileEntry],
    max_size: Option<u64>,
    max_lines: Option<usize>,
    include_binary: bool,
    quiet: bool,
    detailed: bool,
    absolute_paths: bool,
    format: DumpFormat,
    root_path: &Path,
) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if let DumpFormat::Xml = format {
        xml::write_header(&mut handle, root_path)?;
    }

    for file in files {
        let display_path = if absolute_paths {
            file.path.display().to_string()
        } else {
            file.relative.clone()
        };

        let metadata = match std::fs::metadata(&file.path) {
            Ok(m) => m,
            Err(e) => {
                if !quiet {
                    eprintln!("Warning: cannot read metadata for {}: {}", file.relative, e);
                }
                let reason = format!("[ERROR: metadata] {}", e);
                match format {
                    DumpFormat::Plain => {
                        plain::write_marker(&mut handle, &display_path, &reason, None, detailed)?
                    }
                    DumpFormat::Markdown => {
                        markdown::write_marker(&mut handle, &display_path, &reason, None, detailed)?
                    }
                    DumpFormat::Xml => {
                        xml::write_marker(&mut handle, &display_path, &reason, None)?
                    }
                }
                continue;
            }
        };

        let file_size = metadata.len();
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()));

        if file_size == 0 {
            let reason = "[EMPTY FILE]".to_string();
            match format {
                DumpFormat::Plain => plain::write_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Markdown => markdown::write_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Xml => xml::write_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                )?,
            }
            continue;
        }

        if let Some(limit) = max_size {
            if file_size > limit {
                let reason = format!(
                    "[SKIPPED: file size {} > limit {}]",
                    human_readable_size(file_size),
                    human_readable_size(limit)
                );
                match format {
                    DumpFormat::Plain => plain::write_marker(
                        &mut handle,
                        &display_path,
                        &reason,
                        Some((file_size, mtime)),
                        detailed,
                    )?,
                    DumpFormat::Markdown => markdown::write_marker(
                        &mut handle,
                        &display_path,
                        &reason,
                        Some((file_size, mtime)),
                        detailed,
                    )?,
                    DumpFormat::Xml => xml::write_marker(
                        &mut handle,
                        &display_path,
                        &reason,
                        Some((file_size, mtime)),
                    )?,
                }
                continue;
            }
        }

        if !include_binary && is_binary(&file.path, file_size) {
            let reason = "[BINARY FILE SKIPPED]".to_string();
            match format {
                DumpFormat::Plain => plain::write_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Markdown => markdown::write_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Xml => xml::write_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                )?,
            }
            continue;
        }

        let content = read_file_content(&file.path, include_binary)?;
        let content = if let Some(max_lines) = max_lines {
            truncate_lines(&content, max_lines)
        } else {
            content
        };

        match format {
            DumpFormat::Plain => plain::write_file(
                &mut handle,
                &display_path,
                &content,
                Some((file_size, mtime)),
                detailed,
            )?,
            DumpFormat::Markdown => markdown::write_file(
                &mut handle,
                &display_path,
                &content,
                Some((file_size, mtime)),
                detailed,
            )?,
            DumpFormat::Xml => {
                xml::write_file(&mut handle, &display_path, &content, file_size, mtime)?
            }
        }
    }

    if let DumpFormat::Xml = format {
        xml::write_footer(&mut handle)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::utils::{human_readable_size, truncate_lines};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_human_readable_size() {
        assert_eq!(human_readable_size(0), "0 B");
        assert_eq!(human_readable_size(1023), "1023.0 B");
        assert_eq!(human_readable_size(1024), "1.0 KiB");
        assert_eq!(human_readable_size(1536), "1.5 KiB");
        assert_eq!(human_readable_size(1048576), "1.0 MiB");
        assert_eq!(human_readable_size(1073741824), "1.0 GiB");
    }

    #[test]
    fn test_truncate_lines() {
        let s = "a\nb\nc\nd\ne";
        assert_eq!(truncate_lines(s, 3), "a\nb\nc\n[...truncated...]");
        assert_eq!(truncate_lines(s, 5), "a\nb\nc\nd\ne");
        assert_eq!(truncate_lines(s, 10), "a\nb\nc\nd\ne");
    }

    #[test]
    fn test_is_binary() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"hello world").unwrap();
        assert!(!is_binary(&file_path, 11));

        let bin_path = dir.path().join("bin.dat");
        let mut f = File::create(&bin_path).unwrap();
        f.write_all(&[0, 1, 2, 3]).unwrap();
        assert!(is_binary(&bin_path, 4));

        let utf16_path = dir.path().join("utf16.txt");
        let mut f = File::create(&utf16_path).unwrap();
        f.write_all(&[0x68, 0x00, 0x65, 0x00]).unwrap(); // "he" in UTF-16LE
        assert!(is_binary(&utf16_path, 4));
    }

    #[test]
    fn test_lang_from_extension() {
        // Используем markdown::lang_from_extension напрямую
        assert_eq!(markdown::lang_from_extension("main.rs"), "rust");
        assert_eq!(markdown::lang_from_extension("script.py"), "python");
        assert_eq!(markdown::lang_from_extension("README.md"), "markdown");
        assert_eq!(markdown::lang_from_extension("unknown.xyz"), "");
        assert_eq!(markdown::lang_from_extension("no_extension"), "");
    }
}
