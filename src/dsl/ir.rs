use crate::dsl::ast::*;

#[derive(Debug, Clone)]
pub struct IrModule {
    pub name: String,
    pub language: String,
    pub types: Vec<IrType>,
    pub interfaces: Vec<IrInterface>,
    pub functions: Vec<IrFunction>,
    pub tests: Vec<IrTest>,
}

#[derive(Debug, Clone)]
pub enum IrType {
    Struct {
        name: String,
        fields: Vec<IrField>,
    },
    Enum {
        name: String,
        variants: Vec<IrVariant>,
    },
    TypeAlias {
        name: String,
        base: String,
    },
}

#[derive(Debug, Clone)]
pub struct IrField {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone)]
pub struct IrVariant {
    pub name: String,
    pub fields: Vec<IrField>,
}

#[derive(Debug, Clone)]
pub struct IrInterface {
    pub name: String,
    pub methods: Vec<IrMethod>,
}

#[derive(Debug, Clone)]
pub struct IrMethod {
    pub name: String,
    pub params: Vec<IrParam>,
    pub return_type: String,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParam>,
    pub return_type: String,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IrParam {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone)]
pub struct IrTest {
    pub name: String,
    pub expression: String,
}

pub fn ast_to_ir(spec: &Spec) -> IrModule {
    let mut module = IrModule {
        name: String::new(),
        language: "rust".into(),
        types: Vec::new(),
        interfaces: Vec::new(),
        functions: Vec::new(),
        tests: Vec::new(),
    };

    for item in &spec.items {
        match item {
            Item::Project(p) => {
                module.name = p.name.clone();
                module.language = p.language.clone();
            }
            Item::TypeDef(t) => {
                if t.base.to_string().starts_with("struct") {
                    // Parse struct fields from the type ref
                    module.types.push(IrType::Struct {
                        name: t.name.clone(),
                        fields: Vec::new(),
                    });
                } else {
                    module.types.push(IrType::TypeAlias {
                        name: t.name.clone(),
                        base: t.base.to_string(),
                    });
                }
            }
            Item::EnumDef(e) => {
                module.types.push(IrType::Enum {
                    name: e.name.clone(),
                    variants: e.variants.iter().map(|v| IrVariant {
                        name: v.name.clone(),
                        fields: v.fields.iter().map(|f| IrField {
                            name: f.name.clone(),
                            type_name: f.type_ref.to_string(),
                        }).collect(),
                    }).collect(),
                });
            }
            Item::Interface(i) => {
                module.interfaces.push(IrInterface {
                    name: i.name.clone(),
                    methods: i.methods.iter().map(|m| IrMethod {
                        name: m.name.clone(),
                        params: m.params.iter().map(|p| IrParam {
                            name: p.name.clone(),
                            type_name: p.type_ref.to_string(),
                        }).collect(),
                        return_type: m.return_type.to_string(),
                    }).collect(),
                });
            }
            Item::Function(f) => {
                module.functions.push(IrFunction {
                    name: f.name.clone(),
                    params: f.params.iter().map(|p| IrParam {
                        name: p.name.clone(),
                        type_name: p.type_ref.to_string(),
                    }).collect(),
                    return_type: f.return_type.to_string(),
                    constraints: f.constraints.iter().map(|c| format!("{:?}: {}", c.kind, c.value)).collect(),
                });
                for t in &f.tests {
                    module.tests.push(IrTest {
                        name: t.name.clone(),
                        expression: t.expression.clone(),
                    });
                }
            }
            Item::TestDecl(t) => {
                module.tests.push(IrTest {
                    name: t.name.clone(),
                    expression: t.expression.clone(),
                });
            }
            _ => {}
        }
    }

    module
}
