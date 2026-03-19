// src/core/render/long.rs
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
