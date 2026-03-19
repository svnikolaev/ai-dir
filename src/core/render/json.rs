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
    // Сортируем имена дочерних узлов для детерминированного порядка
    let mut child_names: Vec<_> = node.children.keys().collect();
    child_names.sort();
    for name in child_names {
        if let Some(child) = node.children.get(name) {
            children.push(node_to_json(child, depth + 1, max_depth));
        }
    }
    let mut obj = serde_json::Map::new();
    obj.insert("name".into(), json!(node.name));
    if !children.is_empty() {
        obj.insert("children".into(), json!(children));
    }
    if let Some(desc) = &node.description {
        // Для машиночитаемого формата выводим полный список символов
        let full_description = if desc.symbols.is_empty() {
            "no symbols".to_string()
        } else {
            desc.symbols.join(", ")
        };
        obj.insert("description".into(), json!(full_description));
        if let Some(err) = &desc.error {
            obj.insert("error".into(), json!(err));
        }
        // Добавляем отдельное поле symbols для удобства
        obj.insert("symbols".into(), json!(desc.symbols));
    }
    json!(obj)
}
