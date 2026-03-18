use crate::core::types::{Description, OutputFormat};

mod color;
mod json;
mod long;
mod plain;
mod tree;

pub use self::color::print_color;
pub use self::json::print_json;
pub use self::long::print_long_functions;
pub use self::plain::print_plain;
pub use self::tree::build_tree;

pub fn print_tree(
    descriptions: &[Description],
    max_depth: Option<usize>,
    format: OutputFormat,
    show_long: bool,
    no_long_indicator: bool,
) {
    if show_long {
        print_long_functions(descriptions);
        return;
    }

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
        OutputFormat::Plain => print_plain(&tree, "", 0, max_depth, no_long_indicator),
        OutputFormat::Color => print_color(&tree, "", 0, max_depth, no_long_indicator),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Description;
    use std::path::PathBuf;

    #[test]
    fn test_build_tree() {
        let desc = vec![
            Description {
                path: PathBuf::from("/a/b.rs"),
                relative: "a/b.rs".into(),
                text: "file b".into(),
                error: None,
                from_cache: false,
                total_lines: None,
                long_functions: vec![],
            },
            Description {
                path: PathBuf::from("/a/c.rs"),
                relative: "a/c.rs".into(),
                text: "file c".into(),
                error: None,
                from_cache: false,
                total_lines: None,
                long_functions: vec![],
            },
            Description {
                path: PathBuf::from("/d.rs"),
                relative: "d.rs".into(),
                text: "file d".into(),
                error: None,
                from_cache: false,
                total_lines: None,
                long_functions: vec![],
            },
        ];
        let tree = build_tree(&desc);
        assert_eq!(tree.children.len(), 2);
        let a_node = tree.children.get("a").unwrap();
        assert_eq!(a_node.children.len(), 2);
        assert!(a_node.children.contains_key("b.rs"));
        assert!(a_node.children.contains_key("c.rs"));
        let d_node = tree.children.get("d.rs").unwrap();
        assert!(d_node.description.is_some());
    }
}
