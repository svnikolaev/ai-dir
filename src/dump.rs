use crate::types::FileEntry;
use anyhow::Result;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export DumpFormat from main or define here
#[derive(Debug, Clone, Copy)]
pub enum DumpFormat {
    Plain,
    Markdown,
    Xml,
}

/// Основная функция дампа файлов.
pub fn dump_files(
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

    // Для XML нужно записать заголовок и открывающий корневой элемент
    if let DumpFormat::Xml = format {
        write_xml_header(&mut handle, root_path)?;
    }

    for file in files {
        // Определяем путь для вывода (относительный или абсолютный)
        let display_path = if absolute_paths {
            file.path.display().to_string()
        } else {
            file.relative.clone()
        };

        // Получаем метаданные (размер и mtime)
        let metadata = match fs::metadata(&file.path) {
            Ok(m) => m,
            Err(e) => {
                if !quiet {
                    eprintln!("Warning: cannot read metadata for {}: {}", file.relative, e);
                }
                let reason = format!("[ERROR: metadata] {}", e);
                match format {
                    DumpFormat::Plain => {
                        write_plain_marker(&mut handle, &display_path, &reason, None, detailed)?
                    }
                    DumpFormat::Markdown => {
                        write_markdown_marker(&mut handle, &display_path, &reason, None, detailed)?
                    }
                    DumpFormat::Xml => write_xml_marker(&mut handle, &display_path, &reason, None)?,
                }
                continue;
            }
        };

        let file_size = metadata.len();
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()));

        // Проверка на пустой файл
        if file_size == 0 {
            let reason = "[EMPTY FILE]".to_string();
            match format {
                DumpFormat::Plain => write_plain_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Markdown => write_markdown_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Xml => write_xml_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                )?,
            }
            continue;
        }

        // Проверка лимита размера
        if let Some(limit) = max_size {
            if file_size > limit {
                let reason = format!(
                    "[SKIPPED: file size {} > limit {}]",
                    human_readable_size(file_size),
                    human_readable_size(limit)
                );
                match format {
                    DumpFormat::Plain => write_plain_marker(
                        &mut handle,
                        &display_path,
                        &reason,
                        Some((file_size, mtime)),
                        detailed,
                    )?,
                    DumpFormat::Markdown => write_markdown_marker(
                        &mut handle,
                        &display_path,
                        &reason,
                        Some((file_size, mtime)),
                        detailed,
                    )?,
                    DumpFormat::Xml => write_xml_marker(
                        &mut handle,
                        &display_path,
                        &reason,
                        Some((file_size, mtime)),
                    )?,
                }
                continue;
            }
        }

        // Определение бинарности (пропускаем, если не include_binary)
        if !include_binary && is_binary(&file.path, file_size) {
            let reason = "[BINARY FILE SKIPPED]".to_string();
            match format {
                DumpFormat::Plain => write_plain_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Markdown => write_markdown_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                    detailed,
                )?,
                DumpFormat::Xml => write_xml_marker(
                    &mut handle,
                    &display_path,
                    &reason,
                    Some((file_size, mtime)),
                )?,
            }
            continue;
        }

        // Чтение содержимого (возможно, бинарное с заменой)
        let content = read_file_content(&file.path, include_binary)?;
        let content = if let Some(max_lines) = max_lines {
            truncate_lines(&content, max_lines)
        } else {
            content
        };

        // Вывод в зависимости от формата
        match format {
            DumpFormat::Plain => write_plain_file(
                &mut handle,
                &display_path,
                &content,
                Some((file_size, mtime)),
                detailed,
            )?,
            DumpFormat::Markdown => write_markdown_file(
                &mut handle,
                &display_path,
                &content,
                Some((file_size, mtime)),
                detailed,
            )?,
            DumpFormat::Xml => {
                write_xml_file(&mut handle, &display_path, &content, file_size, mtime)?
            }
        }
    }

    // Для XML закрываем корневой элемент
    if let DumpFormat::Xml = format {
        writeln!(handle, "</files>\n</project>")?;
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Вспомогательные функции для чтения и определения бинарности
// -----------------------------------------------------------------------------

/// Проверяет, является ли файл бинарным (содержит нулевой байт в первых 1024 байтах).
fn is_binary(path: &Path, file_size: u64) -> bool {
    if file_size == 0 {
        return false;
    }
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buffer = vec![0; 1024];
    let n = match std::io::Read::read(&mut file, &mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return false,
    };
    buffer[..n].contains(&0)
}

/// Читает файл как строку. Если `include_binary` истинно, читает как байты и заменяет невалидные последовательности.
fn read_file_content(path: &Path, include_binary: bool) -> Result<String> {
    if include_binary {
        let bytes = fs::read(path)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

/// Обрезает содержимое до указанного числа строк.
fn truncate_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let mut result = lines.join("\n");
    if content.lines().count() > max_lines {
        result.push_str("\n[...truncated...]");
    }
    result
}

/// Преобразует размер в человекочитаемый вид.
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

/// Форматирует SystemTime в ISO 8601 (упрощённо).
fn format_time_iso(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

// -----------------------------------------------------------------------------
// Форматтеры для Plain
// -----------------------------------------------------------------------------

fn write_plain_marker<W: Write>(
    handle: &mut W,
    display_path: &str,
    reason: &str,
    meta: Option<(u64, Option<u64>)>, // (size, mtime_secs)
    detailed: bool,
) -> io::Result<()> {
    if detailed {
        if let Some((size, mtime_secs)) = meta {
            let size_str = human_readable_size(size);
            let mtime_str = mtime_secs
                .and_then(|secs| {
                    std::time::UNIX_EPOCH
                        .checked_add(std::time::Duration::from_secs(secs))
                        .map(format_time_iso)
                })
                .unwrap_or_else(|| "unknown".to_string());
            writeln!(
                handle,
                "==== File: {} (size: {}, modified: {}) ====",
                display_path, size_str, mtime_str
            )?;
        } else {
            writeln!(handle, "==== File: {} ====", display_path)?;
        }
    } else {
        writeln!(handle, "==== File: {} ====", display_path)?;
    }
    writeln!(handle, "{}", reason)?;
    writeln!(handle)?; // пустая строка
    Ok(())
}

fn write_plain_file<W: Write>(
    handle: &mut W,
    display_path: &str,
    content: &str,
    meta: Option<(u64, Option<u64>)>,
    detailed: bool,
) -> io::Result<()> {
    if detailed {
        if let Some((size, mtime_secs)) = meta {
            let size_str = human_readable_size(size);
            let mtime_str = mtime_secs
                .and_then(|secs| {
                    std::time::UNIX_EPOCH
                        .checked_add(std::time::Duration::from_secs(secs))
                        .map(format_time_iso)
                })
                .unwrap_or_else(|| "unknown".to_string());
            writeln!(
                handle,
                "==== File: {} (size: {}, modified: {}) ====",
                display_path, size_str, mtime_str
            )?;
        } else {
            writeln!(handle, "==== File: {} ====", display_path)?;
        }
    } else {
        writeln!(handle, "==== File: {} ====", display_path)?;
    }
    write!(handle, "{}", content)?;
    if !content.ends_with('\n') {
        writeln!(handle)?;
    }
    writeln!(handle)?; // пустая строка
    Ok(())
}

// -----------------------------------------------------------------------------
// Форматтеры для Markdown
// -----------------------------------------------------------------------------

/// Возвращает имя языка для блока кода по расширению файла.
fn lang_from_extension(rel_path: &str) -> &'static str {
    let ext = Path::new(rel_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" | "cxx" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "swift" => "swift",
        "kt" => "kotlin",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "md" => "markdown",
        "html" | "htm" => "html",
        "css" => "css",
        "sh" | "bash" => "bash",
        _ => "",
    }
}

fn write_markdown_marker<W: Write>(
    handle: &mut W,
    display_path: &str,
    reason: &str,
    meta: Option<(u64, Option<u64>)>,
    detailed: bool,
) -> io::Result<()> {
    writeln!(handle, "## `{}`", display_path)?;
    if detailed {
        if let Some((size, mtime_secs)) = meta {
            let size_str = human_readable_size(size);
            let mtime_str = mtime_secs
                .and_then(|secs| {
                    std::time::UNIX_EPOCH
                        .checked_add(std::time::Duration::from_secs(secs))
                        .map(format_time_iso)
                })
                .unwrap_or_else(|| "unknown".to_string());
            writeln!(
                handle,
                "\n*Size: {}*, *Modified: {}*\n",
                size_str, mtime_str
            )?;
        }
    }
    writeln!(handle, "```")?;
    writeln!(handle, "{}", reason)?;
    writeln!(handle, "```\n")?;
    Ok(())
}

fn write_markdown_file<W: Write>(
    handle: &mut W,
    display_path: &str,
    content: &str,
    meta: Option<(u64, Option<u64>)>,
    detailed: bool,
) -> io::Result<()> {
    writeln!(handle, "## `{}`", display_path)?;
    if detailed {
        if let Some((size, mtime_secs)) = meta {
            let size_str = human_readable_size(size);
            let mtime_str = mtime_secs
                .and_then(|secs| {
                    std::time::UNIX_EPOCH
                        .checked_add(std::time::Duration::from_secs(secs))
                        .map(format_time_iso)
                })
                .unwrap_or_else(|| "unknown".to_string());
            writeln!(
                handle,
                "\n*Size: {}*, *Modified: {}*\n",
                size_str, mtime_str
            )?;
        }
    }
    let lang = lang_from_extension(display_path);
    if !lang.is_empty() {
        writeln!(handle, "```{}", lang)?;
    } else {
        writeln!(handle, "```")?;
    }
    write!(handle, "{}", content)?;
    if !content.ends_with('\n') {
        writeln!(handle)?;
    }
    writeln!(handle, "```\n")?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Форматтеры для XML
// -----------------------------------------------------------------------------

fn write_xml_header<W: Write>(handle: &mut W, root_path: &Path) -> io::Result<()> {
    let root_absolute = fs::canonicalize(root_path)
        .unwrap_or_else(|_| root_path.to_path_buf())
        .display()
        .to_string();
    let project_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root")
        .to_string();
    let generated = format_time_iso(SystemTime::now());
    writeln!(handle, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        handle,
        r#"<project name="{}" root="{}" generated="{}">"#,
        escape_xml_attr(&project_name),
        escape_xml_attr(&root_absolute),
        escape_xml_attr(&generated)
    )?;
    writeln!(handle, "  <files>")
}

fn write_xml_marker<W: Write>(
    handle: &mut W,
    display_path: &str,
    reason: &str,
    meta: Option<(u64, Option<u64>)>,
) -> io::Result<()> {
    let (size_attr, mtime_attr) = if let Some((size, mtime_secs)) = meta {
        (
            format!(r#" size="{}""#, size),
            mtime_secs.map_or(String::new(), |secs| format!(r#" mtime="{}""#, secs)),
        )
    } else {
        (String::new(), String::new())
    };
    writeln!(
        handle,
        r#"    <file path="{}"{}{} error="{}" />"#,
        escape_xml_attr(display_path),
        size_attr,
        mtime_attr,
        escape_xml_attr(reason)
    )
}

fn write_xml_file<W: Write>(
    handle: &mut W,
    display_path: &str,
    content: &str,
    size: u64,
    mtime: Option<u64>,
) -> io::Result<()> {
    let mtime_attr = mtime.map_or(String::new(), |secs| format!(r#" mtime="{}""#, secs));
    write!(
        handle,
        r#"    <file path="{}" size="{}"{}>"#,
        escape_xml_attr(display_path),
        size,
        mtime_attr
    )?;
    // Оборачиваем содержимое в CDATA, проверяя наличие "]]>"
    if content.contains("]]>") {
        // В реальности крайне редко, но предусмотрим: разбиваем CDATA
        // Простой способ: экранировать как текст (не CDATA) с заменой спецсимволов
        write!(handle, "{}", escape_xml_text(content))?;
    } else {
        write!(handle, "<![CDATA[{}]]>", content)?;
    }
    writeln!(handle, "</file>")
}

/// Экранирование для XML атрибутов (кавычки, &, <, >)
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Экранирование для XML текстового узла (если не CDATA)
fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
        assert!(is_binary(&utf16_path, 4)); // содержит нули -> считается бинарным
    }

    #[test]
    fn test_lang_from_extension() {
        assert_eq!(lang_from_extension("main.rs"), "rust");
        assert_eq!(lang_from_extension("script.py"), "python");
        assert_eq!(lang_from_extension("README.md"), "markdown");
        assert_eq!(lang_from_extension("unknown.xyz"), "");
        assert_eq!(lang_from_extension("no_extension"), "");
    }
}
