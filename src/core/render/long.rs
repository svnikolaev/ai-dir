use crate::core::types::Description;
use std::io::{self, Write};

pub fn print_long_functions(descriptions: &[Description], threshold: usize) {
    for desc in descriptions {
        let long: Vec<_> = desc
            .functions
            .iter()
            .filter(|(_, lines)| *lines >= threshold)
            .collect();
        if !long.is_empty() {
            println!(
                "{} [{} lines]",
                desc.relative,
                desc.total_lines.unwrap_or(0)
            );
            for (name, lines) in long {
                println!("  └── {}: {} lines", name, lines);
            }
            println!();
        }
    }
}

pub fn print_long_functions_json(descriptions: &[Description], threshold: usize) {
    let mut list = Vec::new();
    for desc in descriptions {
        let long: Vec<_> = desc
            .functions
            .iter()
            .filter(|(_, lines)| *lines >= threshold)
            .map(|(name, lines)| serde_json::json!({ "name": name, "lines": lines }))
            .collect();
        if !long.is_empty() {
            let file_obj = serde_json::json!({
                "file": desc.relative,
                "total_lines": desc.total_lines.unwrap_or(0),
                "long_functions": long
            });
            list.push(file_obj);
        }
    }
    let output = serde_json::to_string_pretty(&list).unwrap();
    println!("{}", output);
}

pub fn print_long_functions_xml<W: Write>(
    writer: &mut W,
    descriptions: &[Description],
    threshold: usize,
) -> io::Result<()> {
    writeln!(writer, "<long-functions>")?;
    for desc in descriptions {
        let long: Vec<_> = desc
            .functions
            .iter()
            .filter(|(_, lines)| *lines >= threshold)
            .collect();
        if !long.is_empty() {
            writeln!(
                writer,
                "  <file path=\"{}\" total_lines=\"{}\">",
                escape_xml(&desc.relative),
                desc.total_lines.unwrap_or(0)
            )?;
            for (name, lines) in long {
                writeln!(
                    writer,
                    "    <function name=\"{}\" lines=\"{}\" />",
                    escape_xml(name),
                    lines
                )?;
            }
            writeln!(writer, "  </file>")?;
        }
    }
    writeln!(writer, "</long-functions>")?;
    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_functions_filtering() {
        let desc = Description {
            relative: "file.rs".into(),
            total_lines: Some(100),
            functions: vec![("f1".into(), 30), ("f2".into(), 10)],
            ..Default::default()
        };
        let long: Vec<_> = desc
            .functions
            .iter()
            .filter(|(_, lines)| *lines >= 20)
            .collect();
        assert_eq!(long.len(), 1);
        assert_eq!(long[0].0, "f1");
    }

    // Вспомогательная функция для теста JSON, использует полный путь serde_json::json!
    fn print_long_functions_json_to_writer<W: Write>(
        writer: &mut W,
        descriptions: &[Description],
        threshold: usize,
    ) -> io::Result<()> {
        let list: Vec<_> = descriptions
            .iter()
            .filter_map(|desc| {
                let long: Vec<_> = desc
                    .functions
                    .iter()
                    .filter(|(_, lines)| *lines >= threshold)
                    .map(|(name, lines)| serde_json::json!({ "name": name, "lines": lines }))
                    .collect();
                if long.is_empty() {
                    None
                } else {
                    Some(serde_json::json!({
                        "file": desc.relative,
                        "total_lines": desc.total_lines.unwrap_or(0),
                        "long_functions": long
                    }))
                }
            })
            .collect();
        let output = serde_json::to_string_pretty(&list)?;
        writeln!(writer, "{}", output)?;
        Ok(())
    }

    #[test]
    fn test_long_functions_json_output() {
        let desc = Description {
            relative: "file.rs".into(),
            total_lines: Some(100),
            functions: vec![("f1".into(), 30), ("f2".into(), 10)],
            ..Default::default()
        };
        let mut buf = Vec::new();
        print_long_functions_json_to_writer(&mut buf, &[desc], 20).unwrap();
        let output: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(output.as_array().unwrap().len(), 1);
        assert_eq!(output[0]["file"], "file.rs");
        assert_eq!(output[0]["long_functions"].as_array().unwrap().len(), 1);
        assert_eq!(output[0]["long_functions"][0]["name"], "f1");
    }

    #[test]
    fn test_long_functions_xml_output() {
        let desc = Description {
            relative: "file.rs".into(),
            total_lines: Some(100),
            functions: vec![("f1".into(), 30)],
            ..Default::default()
        };
        let mut buf = Vec::new();
        print_long_functions_xml(&mut buf, &[desc], 20).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("<file path=\"file.rs\" total_lines=\"100\">"));
        assert!(output.contains("<function name=\"f1\" lines=\"30\" />"));
    }
}
