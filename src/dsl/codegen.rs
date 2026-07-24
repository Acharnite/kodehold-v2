use std::path::Path;

use crate::dsl::ir::{IrModule, IrType};

pub fn generate(module: &IrModule, output_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();

    // Generate types file
    let types_code = generate_types(module);
    if !types_code.is_empty() {
        let path = output_dir.join("types.rs");
        std::fs::write(&path, &types_code)?;
        files.push(path.to_string_lossy().to_string());
    }

    // Generate interfaces file
    let interfaces_code = generate_interfaces(module);
    if !interfaces_code.is_empty() {
        let path = output_dir.join("interfaces.rs");
        std::fs::write(&path, &interfaces_code)?;
        files.push(path.to_string_lossy().to_string());
    }

    // Generate functions file
    let functions_code = generate_functions(module);
    if !functions_code.is_empty() {
        let path = output_dir.join("lib.rs");
        // Append to or create lib.rs
        if path.exists() {
            let existing = std::fs::read_to_string(&path)?;
            std::fs::write(&path, format!("{existing}\n{functions_code}"))?;
        } else {
            std::fs::write(&path, &functions_code)?;
        }
        files.push(path.to_string_lossy().to_string());
    }

    // Generate tests
    let tests_code = generate_tests(module);
    if !tests_code.is_empty() {
        let test_dir = output_dir.parent().map(|p| p.join("tests")).unwrap_or_else(|| output_dir.join("../tests"));
        std::fs::create_dir_all(&test_dir)?;
        let path = test_dir.join("spec_tests.rs");
        std::fs::write(&path, &tests_code)?;
        files.push(path.to_string_lossy().to_string());
    }

    Ok(files)
}

fn generate_types(module: &IrModule) -> String {
    let mut code = String::new();

    for ty in &module.types {
        match ty {
            IrType::TypeAlias { name, base } => {
                code.push_str(&format!("pub type {name} = {base};\n"));
            }
            IrType::Struct { name, fields } => {
                code.push_str(&format!("#[derive(Debug, Clone)]\npub struct {name} {{\n"));
                for f in fields {
                    code.push_str(&format!("    pub {}: {},\n", f.name, f.type_name));
                }
                code.push_str("}\n\n");
            }
            IrType::Enum { name, variants } => {
                code.push_str(&format!("#[derive(Debug, Clone, PartialEq)]\npub enum {name} {{\n"));
                for v in variants {
                    if v.fields.is_empty() {
                        code.push_str(&format!("    {},\n", v.name));
                    } else {
                        let fields: Vec<String> = v.fields.iter()
                            .map(|f| format!("{}: {}", f.name, f.type_name))
                            .collect();
                        code.push_str(&format!("    {} {{ {} }},\n", v.name, fields.join(", ")));
                    }
                }
                code.push_str("}\n\n");
            }
        }
    }

    code
}

fn generate_interfaces(module: &IrModule) -> String {
    let mut code = String::new();

    for iface in &module.interfaces {
        code.push_str(&format!("pub trait {} {{\n", iface.name));
        for m in &iface.methods {
            let params: Vec<String> = m.params.iter()
                .map(|p| format!("{}: {}", p.name, p.type_name))
                .collect();
            code.push_str(&format!("    fn {}({}) -> {};\n", m.name, params.join(", "), m.return_type));
        }
        code.push_str("}\n\n");
    }

    code
}

fn generate_functions(module: &IrModule) -> String {
    let mut code = String::new();

    for f in &module.functions {
        let params: Vec<String> = f.params.iter()
            .map(|p| format!("{}: {}", p.name, p.type_name))
            .collect();
        code.push_str(&format!(
            "pub fn {}({}) -> {} {{\n    todo!()\n}}\n\n",
            f.name, params.join(", "), f.return_type
        ));
    }

    code
}

fn generate_tests(module: &IrModule) -> String {
    let mut code = String::new();

    code.push_str("#[cfg(test)]\nmod spec_tests {\n");
    code.push_str("    use super::*;\n\n");

    for t in &module.tests {
        code.push_str(&format!(
            "    #[test]\n    fn test_{}() {{\n        {}\n    }}\n\n",
            t.name.to_lowercase().replace(' ', "_"),
            t.expression
        ));
    }

    code.push_str("}\n");
    code
}
