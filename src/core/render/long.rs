use crate::core::types::Description;

pub fn print_long_functions(descriptions: &[Description]) {
    for desc in descriptions {
        if !desc.long_functions.is_empty() {
            println!(
                "{} [{} lines]",
                desc.relative,
                desc.total_lines.unwrap_or(0)
            );
            for (name, lines) in &desc.long_functions {
                println!("  └── {}: {} lines", name, lines);
            }
            println!();
        }
    }
}
