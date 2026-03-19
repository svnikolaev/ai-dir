use crate::core::types::{Description, OutputFormat};
use std::io::stdout;

mod color;
mod json;
mod long;
mod plain;
mod tree;
mod xml;

pub use self::color::print_color;
pub use self::json::print_json;
pub use self::long::{print_long_functions, print_long_functions_json, print_long_functions_xml};
pub use self::plain::print_plain;
pub use self::tree::build_tree;
pub use self::xml::print_xml;

pub fn print_tree(
    descriptions: &[Description],
    max_depth: Option<usize>,
    format: OutputFormat,
    long_threshold: Option<usize>,
    no_long_indicator: bool,
) {
    if let Some(threshold) = long_threshold {
        match format {
            OutputFormat::Json => print_long_functions_json(descriptions, threshold),
            OutputFormat::Xml => {
                let stdout = stdout();
                let mut handle = stdout.lock();
                print_long_functions_xml(&mut handle, descriptions, threshold).unwrap();
            }
            _ => print_long_functions(descriptions, threshold),
        }
        return;
    }

    let tree = build_tree(descriptions);

    if tree.children.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Xml => println!("<files />"),
            _ => println!("No matching files found."),
        }
        return;
    }

    match format {
        OutputFormat::Json => print_json(&tree, max_depth),
        OutputFormat::Xml => print_xml(&tree, max_depth),
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
                functions: vec![],
                symbols: vec![],
            },
            Description {
                path: PathBuf::from("/a/c.rs"),
                relative: "a/c.rs".into(),
                text: "file c".into(),
                error: None,
                from_cache: false,
                total_lines: None,
                long_functions: vec![],
                functions: vec![],
                symbols: vec![],
            },
            Description {
                path: PathBuf::from("/d.rs"),
                relative: "d.rs".into(),
                text: "file d".into(),
                error: None,
                from_cache: false,
                total_lines: None,
                long_functions: vec![],
                functions: vec![],
                symbols: vec![],
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
