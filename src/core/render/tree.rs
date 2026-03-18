use crate::core::types::Description;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TreeNode {
    pub name: String,
    pub children: HashMap<String, TreeNode>,
    pub description: Option<Description>,
}

impl TreeNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            children: HashMap::new(),
            description: None,
        }
    }
}

pub fn build_tree(descriptions: &[Description]) -> TreeNode {
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
