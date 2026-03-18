use crate::types::FileEntry;
use anyhow::Result;
use std::fs;
use std::io::{self, Write};

/// Публичная функция для вызова из main (пишет в stdout).
pub fn dump_files(
    files: &[FileEntry],
    max_size: Option<u64>,
    max_lines: Option<usize>,
    include_binary: bool,
    quiet: bool,
) -> Result<()> {
    let mut stdout = io::stdout();
    dump_files_to_writer(
        files,
        max_size,
        max_lines,
        include_binary,
        quiet,
        &mut stdout,
    )
}

/// Основная функция дампа, принимает любой Write (удобно для тестирования).
pub fn dump_files_to_writer<W: Write>(
    files: &[FileEntry],
    max_size: Option<u64>,
    max_lines: Option<usize>,
    include_binary: bool,
    quiet: bool,
    writer: &mut W,
) -> Result<()> {
    for file in files {
        let metadata = match fs::metadata(&file.path) {
            Ok(m) => m,
            Err(e) => {
                if !quiet {
                    eprintln!("Warning: cannot read metadata for {}: {}", file.relative, e);
                }
                write_marker(
                    writer,
                    &file.relative,
                    &format!("[ERROR: metadata] {}", e),
                    None,
                )?;
                continue;
            }
        };

        let file_size = metadata.len();

        if file_size == 0 {
            write_marker(writer, &file.relative, "[EMPTY FILE]", Some(file_size))?;
            continue;
        }

        if let Some(limit) = max_size {
            if file_size > limit {
                let reason = format!(
                    "[SKIPPED: file size {} > limit {}]",
                    human_readable_size(file_size),
                    human_readable_size(limit)
                );
                write_marker(writer, &file.relative, &reason, None)?;
                continue;
            }
        }

        let content_result = fs::read_to_string(&file.path);
        match content_result {
            Ok(content) => {
                let content = if let Some(max_lines) = max_lines {
                    truncate_lines(&content, max_lines)
                } else {
                    content
                };
                write_file_content(writer, &file.relative, &content)?;
            }
            Err(e) => {
                if !include_binary {
                    if !quiet {
                        eprintln!("Warning: skipping binary file {}: {}", file.relative, e);
                    }
                    write_marker(
                        writer,
                        &file.relative,
                        "[BINARY FILE SKIPPED]",
                        Some(file_size),
                    )?;
                } else {
                    match fs::read(&file.path) {
                        Ok(bytes) => {
                            let lossy = String::from_utf8_lossy(&bytes);
                            let content = if let Some(max_lines) = max_lines {
                                truncate_lines(&lossy, max_lines)
                            } else {
                                lossy.to_string()
                            };
                            write_file_content(writer, &file.relative, &content)?;
                        }
                        Err(read_err) => {
                            let reason = format!("[ERROR: cannot read] {}", read_err);
                            write_marker(writer, &file.relative, &reason, Some(file_size))?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Записать маркер для файла, содержимое которого не выводится.
fn write_marker<W: Write>(
    writer: &mut W,
    rel_path: &str,
    reason: &str,
    size: Option<u64>,
) -> io::Result<()> {
    let size_str = size.map_or(String::new(), |s| {
        format!(" (size: {})", human_readable_size(s))
    });
    writeln!(writer, "==== File: {} ====", rel_path)?;
    writeln!(writer, "{}{}", reason, size_str)?;
    writeln!(writer)?; // пустая строка
    Ok(())
}

/// Вывести содержимое файла с разделителем.
fn write_file_content<W: Write>(writer: &mut W, rel_path: &str, content: &str) -> io::Result<()> {
    writeln!(writer, "==== File: {} ====", rel_path)?;
    write!(writer, "{}", content)?;
    if !content.ends_with('\n') {
        writeln!(writer)?;
    }
    writeln!(writer)?; // пустая строка после файла
    Ok(())
}

/// Обрезать содержимое до указанного числа строк.
fn truncate_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let mut result = lines.join("\n");
    if content.lines().count() > max_lines {
        result.push_str("\n[...truncated...]");
    }
    result
}

/// Преобразовать размер в байтах в человекочитаемый вид.
fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes == 0 {
        return "0 B".into();
    }
    let i = (bytes as f64).log2() / 10.0;
    let i = i.floor() as usize;
    let i = i.min(UNITS.len() - 1);
    let value = bytes as f64 / (1024u64.pow(i as u32) as f64);
    format!("{:.1} {}", value, UNITS[i])
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_write_marker() {
        let mut buf = Vec::new();
        write_marker(&mut buf, "path/to/file.txt", "[REASON]", Some(12345)).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("==== File: path/to/file.txt ===="));
        assert!(output.contains("[REASON] (size: 12.1 KiB)"));
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_write_file_content() {
        let mut buf = Vec::new();
        write_file_content(&mut buf, "path/to/file.txt", "line1\nline2").unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("==== File: path/to/file.txt ===="));
        assert!(output.contains("line1\nline2"));
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_dump_files_to_writer_text_file() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "line1")?;
        writeln!(file, "line2")?;

        let files = vec![FileEntry {
            path: file_path,
            relative: "test.txt".into(),
        }];

        let mut output = Vec::new();
        dump_files_to_writer(&files, None, None, false, true, &mut output)?;
        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("==== File: test.txt ===="));
        assert!(output_str.contains("line1\nline2"));
        Ok(())
    }

    #[test]
    fn test_dump_files_to_writer_empty_file() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("empty.txt");
        File::create(&file_path)?;

        let files = vec![FileEntry {
            path: file_path,
            relative: "empty.txt".into(),
        }];

        let mut output = Vec::new();
        dump_files_to_writer(&files, None, None, false, true, &mut output)?;
        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("==== File: empty.txt ===="));
        assert!(output_str.contains("[EMPTY FILE] (size: 0 B)"));
        Ok(())
    }

    #[test]
    fn test_dump_files_to_writer_binary_file() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("binary.bin");
        let mut file = File::create(&file_path)?;
        file.write_all(&[0, 159, 146, 150])?;

        let files = vec![FileEntry {
            path: file_path.clone(),
            relative: "binary.bin".into(),
        }];

        // Без include_binary
        let mut output = Vec::new();
        dump_files_to_writer(&files, None, None, false, true, &mut output)?;
        let output_str = String::from_utf8(output)?;
        let metadata = fs::metadata(&file_path)?;
        let expected = format!(
            "[BINARY FILE SKIPPED] (size: {})",
            human_readable_size(metadata.len())
        );
        assert!(output_str.contains(&expected));

        // С include_binary
        let mut output2 = Vec::new();
        dump_files_to_writer(&files, None, None, true, true, &mut output2)?;
        let output2_str = String::from_utf8_lossy(&output2);
        assert!(output2_str.contains("==== File: binary.bin ===="));
        // Проверяем, что содержимое вывелось (хотя бы частично)
        assert!(output2_str.contains("���") || output2_str.contains("\0")); // наличие символов замены
        Ok(())
    }

    #[test]
    fn test_dump_files_to_writer_size_limit() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("large.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "this file is about 30 bytes")?;

        let files = vec![FileEntry {
            path: file_path.clone(),
            relative: "large.txt".into(),
        }];

        let metadata = fs::metadata(&file_path)?;
        let file_size = metadata.len();
        let limit = 10;

        let mut output = Vec::new();
        dump_files_to_writer(&files, Some(limit), None, false, true, &mut output)?;
        let output_str = String::from_utf8(output)?;

        let expected = format!(
            "[SKIPPED: file size {} > limit {}]",
            human_readable_size(file_size),
            human_readable_size(limit)
        );
        assert!(output_str.contains(&expected));
        Ok(())
    }

    #[test]
    fn test_dump_files_to_writer_max_lines() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("lines.txt");
        let mut file = File::create(&file_path)?;
        for i in 1..=10 {
            writeln!(file, "line {}", i)?;
        }

        let files = vec![FileEntry {
            path: file_path,
            relative: "lines.txt".into(),
        }];

        let mut output = Vec::new();
        dump_files_to_writer(&files, None, Some(3), false, true, &mut output)?;
        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("line 1\nline 2\nline 3\n[...truncated...]"));
        Ok(())
    }
}
