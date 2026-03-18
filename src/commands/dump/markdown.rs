use crate::core::utils::{format_time_iso, human_readable_size};
use std::io::{self, Write};
use std::path::Path;

pub(crate) fn lang_from_extension(rel_path: &str) -> &'static str {
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

pub fn write_marker<W: Write>(
    handle: &mut W,
    display_path: &str,
    reason: &str,
    meta: Option<(u64, Option<u64>)>,
    detailed: bool,
) -> io::Result<()> {
    writeln!(handle, "## `{}`", display_path)?;
    writeln!(handle)?; // пустая строка после заголовка
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
            writeln!(handle)?; // пустая строка после метаданных
        }
    }
    writeln!(handle, "```")?;
    writeln!(handle, "{}", reason)?;
    writeln!(handle, "```\n")?;
    Ok(())
}

pub fn write_file<W: Write>(
    handle: &mut W,
    display_path: &str,
    content: &str,
    meta: Option<(u64, Option<u64>)>,
    detailed: bool,
) -> io::Result<()> {
    writeln!(handle, "## `{}`", display_path)?;
    writeln!(handle)?; // пустая строка после заголовка
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
            writeln!(handle)?; // пустая строка после метаданных
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_markdown_write_file_without_detailed() {
        let mut buf = Cursor::new(Vec::new());
        write_file(&mut buf, "path/file.rs", "fn main() {}", None, false).unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        let expected = "## `path/file.rs`\n\n```rust\nfn main() {}\n```\n\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_markdown_write_file_with_detailed() {
        let mut buf = Cursor::new(Vec::new());
        let meta = Some((2048, Some(1700000000)));
        write_file(&mut buf, "path/file.rs", "fn main() {}", meta, true).unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("## `path/file.rs`"));
        assert!(output.contains("*Size: 2.0 KiB*, *Modified:"));
        assert!(output.contains("```rust"));
    }

    #[test]
    fn test_markdown_write_marker() {
        let mut buf = Cursor::new(Vec::new());
        write_marker(&mut buf, "path/file.bin", "[BINARY]", None, false).unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(output, "## `path/file.bin`\n\n```\n[BINARY]\n```\n\n");
    }
}
