use crate::core::render::tree::TreeNode;
use colored::*;

pub fn print_color(
    node: &TreeNode,
    prefix: &str,
    depth: usize,
    max_depth: Option<usize>,
    no_long_indicator: bool,
) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }
    let mut entries: Vec<_> = node.children.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (i, (name, child)) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let new_prefix = if is_last { "    " } else { "│   " };

        if let Some(desc) = &child.description {
            let name_part = if desc.error.is_some() {
                name.red().bold()
            } else {
                name.green().bold()
            };
            let mut parts = Vec::new();
            if let Some(lines) = desc.total_lines {
                parts.push(format!("[{} lines]", lines));
            }
            if !desc.text.is_empty() && desc.text != "no symbols" {
                parts.push(format!("({})", desc.text));
            }
            if !desc.long_functions.is_empty() && !no_long_indicator {
                parts.push(format!("(long: {})", desc.long_functions.len()));
            }
            let extra = if parts.is_empty() {
                String::new()
            } else {
                format!(" {}", parts.join(" "))
            };
            println!(
                "{}{}{}{}",
                prefix.bright_black(),
                connector.bright_black(),
                name_part,
                extra
            );
        } else {
            println!(
                "{}{}{}/",
                prefix.bright_black(),
                connector.bright_black(),
                name.cyan().bold()
            );
        }
        print_color(
            child,
            &(prefix.to_owned() + new_prefix),
            depth + 1,
            max_depth,
            no_long_indicator,
        );
    }
}
