use crate::core::utils::format_time_iso;
use crate::core::utils::human_readable_size;
use std::io::{self, Write};

pub fn write_marker<W: Write>(
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

pub fn write_file<W: Write>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_plain_write_file_without_detailed() {
        let mut buf = Cursor::new(Vec::new());
        write_file(&mut buf, "path/to/file.txt", "line1\nline2", None, false).unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        let expected = "==== File: path/to/file.txt ====\nline1\nline2\n\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_plain_write_file_with_detailed() {
        let mut buf = Cursor::new(Vec::new());
        let mtime = 1700000000;
        let meta = Some((1024, Some(mtime)));
        write_file(&mut buf, "path/to/file.txt", "content", meta, true).unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains("==== File: path/to/file.txt (size: 1.0 KiB, modified:"));
        assert!(output.contains(") ===="));
        assert!(output.contains("content"));
    }

    #[test]
    fn test_plain_write_marker() {
        let mut buf = Cursor::new(Vec::new());
        write_marker(&mut buf, "path/to/file.txt", "[ERROR: test]", None, false).unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert_eq!(
            output,
            "==== File: path/to/file.txt ====\n[ERROR: test]\n\n"
        );
    }
}
