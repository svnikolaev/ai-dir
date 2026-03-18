use crate::core::utils::format_time_iso;
use std::io::{self, Write};
use std::path::Path;
use std::time::SystemTime;

fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn write_header<W: Write>(handle: &mut W, root_path: &Path) -> io::Result<()> {
    let root_canonical =
        std::fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let project_name = root_canonical
        .components()
        .last()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("project")
        .to_string();
    let generated = format_time_iso(SystemTime::now());
    writeln!(handle, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        handle,
        r#"<project name="{}" generated="{}">"#,
        escape_xml_attr(&project_name),
        escape_xml_attr(&generated)
    )?;
    writeln!(handle, "  <files>")
}

pub fn write_marker<W: Write>(
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

pub fn write_file<W: Write>(
    handle: &mut W,
    display_path: &str,
    content: &str,
    size: u64,
    mtime: Option<u64>,
) -> io::Result<()> {
    let mtime_attr = mtime.map_or(String::new(), |secs| format!(r#" mtime="{}""#, secs));
    writeln!(
        handle,
        r#"    <file path="{}" size="{}"{}>"#,
        escape_xml_attr(display_path),
        size,
        mtime_attr
    )?;
    // Отступ для CDATA (читаемость)
    write!(handle, "      ")?;
    if content.contains("]]>") {
        write!(handle, "{}", escape_xml_text(content))?;
    } else {
        write!(handle, "<![CDATA[{}]]>", content)?;
    }
    writeln!(handle)?; // перевод строки после CDATA
    writeln!(handle, "    </file>")?;
    Ok(())
}

pub fn write_footer<W: Write>(handle: &mut W) -> io::Result<()> {
    writeln!(handle, "  </files>\n</project>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_xml_write_file() {
        let mut buf = Cursor::new(Vec::new());
        write_file(
            &mut buf,
            "path/file.rs",
            "fn main() {}",
            1234,
            Some(1700000000),
        )
        .unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        // Проверяем наличие ключевых частей, игнорируя форматирование пробелами
        assert!(output.contains(r#"<file path="path/file.rs" size="1234" mtime="1700000000">"#));
        assert!(output.contains("<![CDATA[fn main() {}]]>"));
        assert!(output.contains("</file>"));
    }

    #[test]
    fn test_xml_write_marker() {
        let mut buf = Cursor::new(Vec::new());
        write_marker(
            &mut buf,
            "path/file.bin",
            "[BINARY]",
            Some((999, Some(1700000000))),
        )
        .unwrap();
        let output = String::from_utf8(buf.into_inner()).unwrap();
        assert!(output.contains(
            r#"<file path="path/file.bin" size="999" mtime="1700000000" error="[BINARY]" />"#
        ));
    }

    #[test]
    fn test_xml_escape_attr() {
        assert_eq!(
            escape_xml_attr("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }
}
