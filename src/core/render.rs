use crate::core::types::{Description, OutputFormat};
use colored::*;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug)]
struct TreeNode {
    name: String,
    children: HashMap<String, TreeNode>,
    description: Option<Description>,
}

impl TreeNode {
    fn new(name: String) -> Self {
        Self {
            name,
            children: HashMap::new(),
            description: None,
        }
    }
}

fn build_tree(descriptions: &[Description]) -> TreeNode {
    let mut root = TreeNode::new("".into());

    for desc in descriptions {
        let parts: Vec<&str> = desc.relative.split('/').collect();
        let mut node = &mut root;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // file part
                node = node
                    .children
                    .entry(part.to_string())
                    .or_insert_with(|| TreeNode::new(part.to_string()));
                node.description = Some(desc.clone());
            } else {
                node = node
                    .children
                    .entry(part.to_string())
                    .or_insert_with(|| TreeNode::new(part.to_string()));
            }
        }
    }
    root
}

pub fn print_tree(descriptions: &[Description], max_depth: Option<usize>, format: OutputFormat) {
    let tree = build_tree(descriptions);

    if tree.children.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            _ => println!("No matching files found."),
        }
        return;
    }

    match format {
        OutputFormat::Json => print_json(&tree, max_depth),
        OutputFormat::Plain => print_plain(&tree, "", 0, max_depth),
        OutputFormat::Color => print_color(&tree, "", 0, max_depth),
    }
}

fn print_json(node: &TreeNode, max_depth: Option<usize>) {
    let value = node_to_json(node, 1, max_depth);
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}

fn node_to_json(node: &TreeNode, depth: usize, max_depth: Option<usize>) -> serde_json::Value {
    if let Some(max) = max_depth {
        if depth > max {
            return json!({ "name": node.name, "truncated": true });
        }
    }
    let mut children = Vec::new();
    for child in node.children.values() {
        children.push(node_to_json(child, depth + 1, max_depth));
    }
    let mut obj = serde_json::Map::new();
    obj.insert("name".into(), json!(node.name));
    if !children.is_empty() {
        obj.insert("children".into(), json!(children));
    }
    if let Some(desc) = &node.description {
        obj.insert("description".into(), json!(desc.text));
        if let Some(err) = &desc.error {
            obj.insert("error".into(), json!(err));
        }
    }
    json!(obj)
}

fn print_plain(node: &TreeNode, prefix: &str, depth: usize, max_depth: Option<usize>) {
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
            // Оборачиваем описание в скобки
            format!("{}{}{}  ({})", prefix, connector, name, desc.text)
        } else {
            format!("{}{}{}/", prefix, connector, name)
        };
        println!("{}", line);
        print_plain(
            child,
            &(prefix.to_owned() + new_prefix),
            depth + 1,
            max_depth,
        );
    }
}

fn print_color(node: &TreeNode, prefix: &str, depth: usize, max_depth: Option<usize>) {
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
            let summary = if desc.error.is_some() {
                desc.text.red().to_string()
            } else {
                desc.text.clone()
            };
            // Оборачиваем описание в скобки
            println!(
                "{}{}{}  ({})",
                prefix.bright_black(),
                connector.bright_black(),
                name_part,
                summary
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
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Description;

    #[test]
    fn test_build_tree() {
        let desc = vec![
            Description {
                path: "/a/b.rs".into(),
                relative: "a/b.rs".into(),
                text: "file b".into(),
                error: None,
                from_cache: false,
            },
            Description {
                path: "/a/c.rs".into(),
                relative: "a/c.rs".into(),
                text: "file c".into(),
                error: None,
                from_cache: false,
            },
            Description {
                path: "/d.rs".into(),
                relative: "d.rs".into(),
                text: "file d".into(),
                error: None,
                from_cache: false,
            },
        ];
        let tree = build_tree(&desc);
        assert_eq!(tree.children.len(), 2); // "a" и "d.rs"
        let a_node = tree.children.get("a").unwrap();
        assert_eq!(a_node.children.len(), 2); // b.rs, c.rs
        assert!(a_node.children.contains_key("b.rs"));
        assert!(a_node.children.contains_key("c.rs"));
        let d_node = tree.children.get("d.rs").unwrap();
        assert!(d_node.description.is_some());
    }

    // Дополнительно можно протестировать print_json и т.д., но они используют stdout
}
