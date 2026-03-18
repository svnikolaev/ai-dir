use crate::core::render::tree::TreeNode;

pub fn print_plain(
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

        let line = if let Some(desc) = &child.description {
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
            format!("{}{}{}{}", prefix, connector, name, extra)
        } else {
            format!("{}{}{}/", prefix, connector, name)
        };
        println!("{}", line);
        print_plain(
            child,
            &(prefix.to_owned() + new_prefix),
            depth + 1,
            max_depth,
            no_long_indicator,
        );
    }
}
