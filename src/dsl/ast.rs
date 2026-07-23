use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Project(ProjectDecl),
    TypeDef(TypeDef),
    EnumDef(EnumDef),
    Interface(InterfaceDef),
    Function(FunctionSpec),
    ConstraintDecl(ConstraintDecl),
    TestDecl(TestDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectDecl {
    pub name: String,
    pub version: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub base: TypeRef,
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDef {
    pub name: String,
    pub methods: Vec<MethodSig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodSig {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSpec {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub constraints: Vec<Constraint>,
    pub tests: Vec<TestDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_ref: TypeRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Named(String),
    Vec(Box<TypeRef>),
    Result(Box<TypeRef>, Box<TypeRef>),
    Option(Box<TypeRef>),
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRef::Named(n) => write!(f, "{n}"),
            TypeRef::Vec(t) => write!(f, "Vec<{t}>"),
            TypeRef::Result(ok, err) => write!(f, "Result<{ok}, {err}>"),
            TypeRef::Option(t) => write!(f, "Option<{t}>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub kind: ConstraintKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    TimeComplexity,
    SpaceComplexity,
    Purity,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintDecl {
    pub target: String,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestDecl {
    pub name: String,
    pub expression: String,
}
