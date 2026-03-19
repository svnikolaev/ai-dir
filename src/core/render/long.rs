use crate::core::types::Description;
use std::io::{self, Write};

pub fn print_long_functions(descriptions: &[Description]) {
    for desc in descriptions {
        if !desc.long_functions.is_empty() {
            println!(
                "{} [{} lines]",
                desc.relative,
                desc.total_lines.unwrap_or(0)
            );
            for (name, lines) in &desc.long_functions {
                println!("  └── {}: {} lines", name, lines);
            }
            println!();
        }
    }
}

pub fn print_long_functions_json(descriptions: &[Description]) {
    let mut list = Vec::new();
    for desc in descriptions {
        if !desc.long_functions.is_empty() {
            let file_obj = serde_json::json!({
                "file": desc.relative,
                "total_lines": desc.total_lines.unwrap_or(0),
                "long_functions": desc.long_functions.iter().map(|(name, lines)| {
                    serde_json::json!({ "name": name, "lines": lines })
                }).collect::<Vec<_>>()
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
) -> io::Result<()> {
    writeln!(writer, "<long-functions>")?;
    for desc in descriptions {
        if !desc.long_functions.is_empty() {
            writeln!(
                writer,
                "  <file path=\"{}\" total_lines=\"{}\">",
                escape_xml(&desc.relative),
                desc.total_lines.unwrap_or(0)
            )?;
            for (name, lines) in &desc.long_functions {
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
