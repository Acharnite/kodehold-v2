pub mod ast;
pub mod codegen;
pub mod ir;
pub mod parser;

use std::path::Path;

pub fn compile_spec(input: &Path, output_dir: &Path) -> anyhow::Result<CompileResult> {
    let content = std::fs::read_to_string(input)?;
    let mut p = parser::Parser::new(&content);
    let spec = p.parse_spec().map_err(|e| anyhow::anyhow!("{e}"))?;
    let ir_module = ir::ast_to_ir(&spec);
    let files = codegen::generate(&ir_module, output_dir)?;
    Ok(CompileResult {
        module_name: ir_module.name,
        language: ir_module.language,
        files,
        types: ir_module.types.len(),
        functions: ir_module.functions.len(),
        interfaces: ir_module.interfaces.len(),
        tests: ir_module.tests.len(),
    })
}

pub struct CompileResult {
    pub module_name: String,
    pub language: String,
    pub files: Vec<String>,
    pub types: usize,
    pub functions: usize,
    pub interfaces: usize,
    pub tests: usize,
}
