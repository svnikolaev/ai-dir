use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{
    File, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, ItemType, Type,
    visit::Visit,
};

// Пустая функция для совместимости с pattern.rs (Rust больше не использует регулярки)
pub fn patterns(_map: &mut std::collections::HashMap<&'static str, super::LanguagePatterns>) {
    // Ничего не делаем, так как Rust обрабатывается через syn
}

#[derive(Debug, Default)]
pub struct RustSymbolCollector {
    pub symbols: Vec<String>,            // все объявления (для отображения)
    pub functions: Vec<(String, usize)>, // функции с размерами
}

impl<'ast> Visit<'ast> for RustSymbolCollector {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let name = node.ident.to_string();
        self.symbols.push(format!("struct {}", name));
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let name = node.ident.to_string();
        self.symbols.push(format!("enum {}", name));
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        let end_line = node.span().end().line;
        let lines = end_line - start_line + 1;
        self.functions.push((name.clone(), lines));
        self.symbols.push(format!("fn {}", name));
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        let name = node.sig.ident.to_string();
        let start_line = node.span().start().line;
        let end_line = node.span().end().line;
        let lines = end_line - start_line + 1;
        self.functions.push((name.clone(), lines));
        self.symbols.push(format!("fn {}", name));
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if let Some((_, trait_path, _)) = &node.trait_ {
            // Извлекаем имя трейта (последний сегмент пути)
            let trait_name = if let Some(segment) = trait_path.segments.last() {
                segment.ident.to_string()
            } else {
                quote!(#trait_path).to_string()
            };
            self.symbols.push(format!("impl {}", trait_name));
        } else {
            // Извлекаем имя типа (последний сегмент пути, если это Type::Path)
            let self_ty_name = if let Type::Path(type_path) = &*node.self_ty {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident.to_string()
                } else {
                    quote!(#node.self_ty).to_string()
                }
            } else {
                quote!(#node.self_ty).to_string()
            };
            self.symbols.push(format!("impl {}", self_ty_name));
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        let name = node.ident.to_string();
        self.symbols.push(format!("trait {}", name));
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast ItemType) {
        let name = node.ident.to_string();
        self.symbols.push(format!("type {}", name));
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast ItemConst) {
        let name = node.ident.to_string();
        self.symbols.push(format!("const {}", name));
        syn::visit::visit_item_const(self, node);
    }
}

pub fn extract_from_rust(content: &str) -> (Vec<String>, Vec<(String, usize)>) {
    let ast: File = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(_) => return (Vec::new(), Vec::new()),
    };
    let mut collector = RustSymbolCollector::default();
    collector.visit_file(&ast);
    // Убираем дубликаты в символах
    let mut seen = HashSet::new();
    let symbols = collector
        .symbols
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
        .collect();
    (symbols, collector.functions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_parser() {
        let code = r#"
            struct MyStruct;
            enum MyEnum { A, B }
            fn my_function() {}
            impl MyStruct {}
            trait MyTrait {}
            type MyType = i32;
            const MY_CONST: i32 = 42;
        "#;
        let (symbols, functions) = extract_from_rust(code);

        // Для отладки можно раскомментировать:
        // println!("symbols: {:#?}", symbols);

        assert!(symbols.iter().any(|s| s == "struct MyStruct"));
        assert!(symbols.iter().any(|s| s == "enum MyEnum"));
        assert!(symbols.iter().any(|s| s == "fn my_function"));
        assert!(symbols.iter().any(|s| s == "impl MyStruct"));
        assert!(symbols.iter().any(|s| s == "trait MyTrait"));
        assert!(symbols.iter().any(|s| s == "type MyType"));
        assert!(symbols.iter().any(|s| s == "const MY_CONST"));
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].0, "my_function");
    }
}
