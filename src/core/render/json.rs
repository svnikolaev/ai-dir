use crate::core::render::tree::TreeNode;
use serde_json::json;

pub fn print_json(node: &TreeNode, max_depth: Option<usize>) {
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
