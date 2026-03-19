use crate::core::render::tree::TreeNode;
use std::io::{self, Write};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn print_xml(node: &TreeNode, max_depth: Option<usize>) {
    let mut buffer = Vec::new();
    write_xml_node(&mut buffer, node, 0, max_depth).unwrap();
    println!("{}", String::from_utf8(buffer).unwrap());
}

fn write_xml_node<W: Write>(
    writer: &mut W,
    node: &TreeNode,
    depth: usize,
    max_depth: Option<usize>,
) -> io::Result<()> {
    if let Some(max) = max_depth {
        if depth > max {
            writeln!(
                writer,
                "<file name=\"{}\" truncated=\"true\" />",
                escape_xml(&node.name)
            )?;
            return Ok(());
        }
    }

    if node.children.is_empty() && node.description.is_none() {
        writeln!(writer, "<file name=\"{}\" />", escape_xml(&node.name))?;
        return Ok(());
    }

    if node.children.is_empty() {
        // Файл
        if let Some(desc) = &node.description {
            let full_description = if desc.symbols.is_empty() {
                "no symbols".to_string()
            } else {
                desc.symbols.join(", ")
            };
            writeln!(
                writer,
                "<file name=\"{}\" description=\"{}\" long_functions=\"{}\" total_lines=\"{}\" />",
                escape_xml(&node.name),
                escape_xml(&full_description),
                desc.long_functions.len(),
                desc.total_lines.unwrap_or(0)
            )?;
        } else {
            writeln!(writer, "<file name=\"{}\" />", escape_xml(&node.name))?;
        }
    } else {
        // Директория
        writeln!(writer, "<directory name=\"{}\">", escape_xml(&node.name))?;
        let mut children: Vec<_> = node.children.values().collect();
        children.sort_by_key(|c| c.name.clone());
        for child in children {
            write_xml_node(writer, child, depth + 1, max_depth)?;
        }
        writeln!(writer, "</directory>")?;
    }
    Ok(())
}
