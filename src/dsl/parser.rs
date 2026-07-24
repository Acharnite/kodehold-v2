use crate::dsl::ast::*;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.message, self.line, self.col)
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    src: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            src: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn parse_spec(&mut self) -> Result<Spec, ParseError> {
        let mut items = Vec::new();
        self.skip_whitespace();
        while self.pos < self.src.len() {
            if self.peek() == Some('/') && self.peek_at(1) == Some('/') {
                self.skip_line_comment();
                continue;
            }
            if self.peek() == Some('/') && self.peek_at(1) == Some('*') {
                self.skip_block_comment();
                continue;
            }
            let item = self.parse_item()?;
            items.push(item);
            self.skip_whitespace();
        }
        Ok(Spec { items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let word = self.parse_word()?;
        match word.as_str() {
            "project" => {
                let name = self.parse_string()?;
                self.expect_char('{')?;
                let (version, language) = self.parse_project_body()?;
                self.expect_char('}')?;
                Ok(Item::Project(ProjectDecl { name, version, language }))
            }
            "type" => {
                let name = self.parse_word()?;
                self.expect_char('=')?;
                let base = self.parse_type_ref()?;
                let constraint = if self.peek() == Some(',') {
                    self.advance();
                    Some(self.parse_until_eol())
                } else {
                    None
                };
                Ok(Item::TypeDef(TypeDef { name, base, constraint }))
            }
            "enum" => {
                let name = self.parse_word()?;
                self.expect_char('{')?;
                let mut variants = Vec::new();
        self.skip_whitespace();
        while self.peek() != Some('}') && self.pos < self.src.len() {
            let v = self.parse_variant()?;
            variants.push(v);
            self.skip_whitespace();
            if self.peek() == Some(',') { self.advance(); }
            self.skip_whitespace();
        }
                self.expect_char('}')?;
                Ok(Item::EnumDef(EnumDef { name, variants }))
            }
            "interface" => {
                let name = self.parse_word()?;
                self.expect_char('{')?;
                let mut methods = Vec::new();
                self.skip_whitespace();
                while self.peek() != Some('}') && self.pos < self.src.len() {
                    let m = self.parse_method_sig()?;
                    methods.push(m);
                    self.skip_whitespace();
                }
                self.expect_char('}')?;
                Ok(Item::Interface(InterfaceDef { name, methods }))
            }
            "fn" => {
                let name = self.parse_word()?;
                self.expect_char('(')?;
                let mut params = Vec::new();
                self.skip_whitespace();
                while self.peek() != Some(')') && self.pos < self.src.len() {
                    let p = self.parse_param()?;
                    params.push(p);
                    self.skip_whitespace();
                    if self.peek() == Some(',') {
                        self.advance();
                        self.skip_whitespace();
                    }
                }
                self.expect_char(')')?;
                self.skip_whitespace();
                let return_type = if self.peek() == Some('-') && self.peek_at(1) == Some('>') {
                    self.advance(); self.advance();
                    self.skip_whitespace();
                    self.parse_type_ref()?
                } else {
                    TypeRef::Named("()".into())
                };
                let mut constraints = Vec::new();
                let mut tests = Vec::new();
                self.skip_whitespace();
                if self.peek() == Some('{') {
                    self.advance();
                    self.skip_whitespace();
                    loop {
                        if self.peek() == Some('}') { self.advance(); break; }
                        if self.peek() == Some('/') && self.peek_at(1) == Some('/') {
                            self.skip_line_comment();
                            continue;
                        }
                        let keyword = self.parse_word()?;
                        match keyword.as_str() {
                            "constraint" => {
                                self.expect_char('{')?;
                                self.skip_whitespace();
                                while self.peek() != Some('}') && self.pos < self.src.len() {
                                    let c = self.parse_constraint_line()?;
                                    constraints.push(c);
                                    self.skip_whitespace();
                                }
                                self.expect_char('}')?;
                            }
                            "test" => {
                                let name = self.parse_word()?;
                                self.expect_char(':')?;
                                self.expect_char('{')?;
                                let expr = self.parse_until('}');
                                self.expect_char('}')?;
                                tests.push(TestDecl { name, expression: expr });
                            }
                            _ => {}
                        }
                        self.skip_whitespace();
                    }
                }
                Ok(Item::Function(FunctionSpec { name, params, return_type, constraints, tests }))
            }
            "constraint" => {
                let target = self.parse_word()?;
                self.expect_char(':')?;
                let mut constraints = Vec::new();
                self.skip_whitespace();
                if self.peek() == Some('{') {
                    self.advance();
                    self.skip_whitespace();
                    while self.peek() != Some('}') && self.pos < self.src.len() {
                        let c = self.parse_constraint_line()?;
                        constraints.push(c);
                        self.skip_whitespace();
                    }
                    self.expect_char('}')?;
                } else {
                    let v = self.parse_until_eol();
                    constraints.push(Constraint { kind: ConstraintKind::Custom(target.clone()), value: v });
                }
                Ok(Item::ConstraintDecl(ConstraintDecl { target, constraints }))
            }
            "test" => {
                let name = self.parse_word()?;
                self.expect_char(':')?;
                self.expect_char('{')?;
                let expr = self.parse_until('}');
                self.expect_char('}')?;
                Ok(Item::TestDecl(TestDecl { name, expression: expr }))
            }
            other => Err(self.error(format!("unexpected keyword: {other}"))),
        }
    }

    fn parse_project_body(&mut self) -> Result<(String, String), ParseError> {
        let mut version = String::new();
        let mut language = String::new();
        self.skip_whitespace();
        loop {
            if self.peek() == Some('}') { break; }
            let key = self.parse_word()?;
            self.expect_char('=')?;
            let val = self.parse_string()?;
            match key.as_str() {
                "version" => version = val,
                "language" => language = val,
                _ => {}
            }
            self.skip_whitespace();
            if self.peek() == Some(',') { self.advance(); }
            self.skip_whitespace();
        }
        Ok((version, language))
    }

    fn parse_variant(&mut self) -> Result<Variant, ParseError> {
        let name = self.parse_word()?;
        self.skip_whitespace();
        let fields = if self.peek() == Some('(') || self.peek() == Some('{') {
            self.advance();
            let mut f = Vec::new();
            self.skip_whitespace();
            while self.peek() != Some(')') && self.peek() != Some('}') && self.pos < self.src.len() {
                let field = self.parse_field()?;
                f.push(field);
                self.skip_whitespace();
                if self.peek() == Some(',') { self.advance(); }
                self.skip_whitespace();
            }
            if self.peek() == Some(')') || self.peek() == Some('}') { self.advance(); }
            f
        } else {
            vec![]
        };
        Ok(Variant { name, fields })
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let name = self.parse_word()?;
        self.expect_char(':')?;
        let type_ref = self.parse_type_ref()?;
        Ok(Field { name, type_ref })
    }

    fn parse_method_sig(&mut self) -> Result<MethodSig, ParseError> {
        self.expect_keyword("fn")?;
        let name = self.parse_word()?;
        self.expect_char('(')?;
        let mut params = Vec::new();
        self.skip_whitespace();
        while self.peek() != Some(')') && self.pos < self.src.len() {
            let p = self.parse_param()?;
            params.push(p);
            self.skip_whitespace();
            if self.peek() == Some(',') { self.advance(); self.skip_whitespace(); }
        }
        self.expect_char(')')?;
        self.skip_whitespace();
        let return_type = if self.peek() == Some('-') && self.peek_at(1) == Some('>') {
            self.advance(); self.advance();
            self.skip_whitespace();
            self.parse_type_ref()?
        } else {
            TypeRef::Named("()".into())
        };
        Ok(MethodSig { name, params, return_type })
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = self.parse_word()?;
        self.expect_char(':')?;
        let type_ref = self.parse_type_ref()?;
        Ok(Param { name, type_ref })
    }

    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        let name = self.parse_word()?;
        match name.as_str() {
            "Vec" => {
                self.expect_char('<')?;
                let inner = self.parse_type_ref()?;
                self.expect_char('>')?;
                Ok(TypeRef::Vec(Box::new(inner)))
            }
            "Result" => {
                self.expect_char('<')?;
                let ok = self.parse_type_ref()?;
                self.skip_whitespace();
                self.expect_char(',')?;
                self.skip_whitespace();
                let err = self.parse_type_ref()?;
                self.expect_char('>')?;
                Ok(TypeRef::Result(Box::new(ok), Box::new(err)))
            }
            "Option" => {
                self.expect_char('<')?;
                let inner = self.parse_type_ref()?;
                self.expect_char('>')?;
                Ok(TypeRef::Option(Box::new(inner)))
            }
            "struct" => {
                self.expect_char('{')?;
                self.skip_whitespace();
                // Consume everything until closing brace
                while let Some(c) = self.peek() {
                    if c == '}' { break; }
                    self.advance();
                }
                self.expect_char('}')?;
                // Return a named type representing the struct
                Ok(TypeRef::Named("struct".into()))
            }
            _ => Ok(TypeRef::Named(name)),
        }
    }

    fn parse_constraint_line(&mut self) -> Result<Constraint, ParseError> {
        let kw = self.parse_word()?;
        self.expect_char(':')?;
        self.skip_whitespace();
        let value = self.parse_constraint_value();
        let kind = match kw.as_str() {
            "time" => ConstraintKind::TimeComplexity,
            "space" => ConstraintKind::SpaceComplexity,
            "pure" => { return Ok(Constraint { kind: ConstraintKind::Purity, value: "true".into() }); }
            other => ConstraintKind::Custom(other.into()),
        };
        Ok(Constraint { kind, value })
    }

    // Lexer utilities
    fn parse_word(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(self.error("expected identifier".into()));
        }
        Ok(self.src[start..self.pos].iter().collect())
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.skip_whitespace();
        self.expect_char('"')?;
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '"' { break; }
            self.advance();
        }
        let value: String = self.src[start..self.pos].iter().collect();
        self.expect_char('"')?;
        Ok(value)
    }

    fn parse_until_eol(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '\n' { break; }
            self.advance();
        }
        self.src[start..self.pos].iter().collect()
    }

    fn parse_until(&mut self, _delimiter: char) -> String {
        let start = self.pos;
        let mut depth = 1;
        while let Some(c) = self.peek() {
            if c == '{' { depth += 1; }
            if c == '}' { depth -= 1; }
            if depth == 0 { break; }
            self.advance();
        }
        let end = if depth == 0 { self.pos - 1 } else { self.pos };
        self.src[start..end].iter().collect()
    }

    fn parse_constraint_value(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_whitespace() || c == '}' { break; }
            self.advance();
        }
        self.src[start..self.pos].iter().collect()
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), ParseError> {
        let word = self.parse_word()?;
        if word != kw {
            return Err(self.error(format!("expected keyword '{kw}', got '{word}'")));
        }
        Ok(())
    }

    fn expect_char(&mut self, ch: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        if self.peek() == Some(ch) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("expected '{ch}'")))
        }
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.src.get(self.pos + offset).copied()
    }

    fn advance(&mut self) {
        if self.peek() == Some('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' { self.advance(); break; }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance(); self.advance(); // skip /*
        while let Some(c) = self.peek() {
            if c == '*' && self.peek_at(1) == Some('/') { self.advance(); self.advance(); break; }
            self.advance();
        }
    }

    fn error(&self, msg: String) -> ParseError {
        ParseError { message: msg, line: self.line, col: self.col }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_alias() {
        let mut p = Parser::new("type Price = f64");
        let spec = p.parse_spec().unwrap();
        assert_eq!(spec.items.len(), 1);
        if let Item::TypeDef(t) = &spec.items[0] {
            assert_eq!(t.name, "Price");
            assert_eq!(t.base.to_string(), "f64");
        } else {
            panic!("expected TypeDef");
        }
    }

    #[test]
    fn test_parse_enum() {
        let input = "enum Regime { Trending, Ranging { upper: Price, lower: Price }, Volatile }";
        let mut p = Parser::new(input);
        let spec = p.parse_spec().unwrap();
        assert_eq!(spec.items.len(), 1);
        if let Item::EnumDef(e) = &spec.items[0] {
            assert_eq!(e.name, "Regime");
            assert_eq!(e.variants.len(), 3);
        } else {
            panic!("expected EnumDef");
        }
    }

    #[test]
    fn test_parse_interface() {
        let input = "interface MarketDataFeed { fn fetch(symbol: String, from: u64) -> Result<Vec<OHLCV>, String> }";
        let mut p = Parser::new(input);
        let spec = p.parse_spec().unwrap();
        assert_eq!(spec.items.len(), 1);
        if let Item::Interface(i) = &spec.items[0] {
            assert_eq!(i.methods.len(), 1);
            assert_eq!(i.methods[0].params.len(), 2);
        } else {
            panic!("expected Interface");
        }
    }

    #[test]
    fn test_parse_function_with_constraints() {
        let input = "fn classify(data: Vec<OHLCV>) -> Regime {
            constraint { time: O(n) space: O(1) pure: true }
            test trending_up: { assert classify(mock_trending()) == Trending }
        }";
        let mut p = Parser::new(input);
        let spec = p.parse_spec().unwrap();
        assert_eq!(spec.items.len(), 1);
        if let Item::Function(f) = &spec.items[0] {
            assert_eq!(f.name, "classify");
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.constraints.len(), 3);
            assert_eq!(f.tests.len(), 1);
        } else {
            panic!("expected Function");
        }
    }

    #[test]
    fn test_parse_project() {
        let input = "project \"my-bot\" { version = \"0.1.0\", language = \"rust\" }";
        let mut p = Parser::new(input);
        let spec = p.parse_spec().unwrap();
        assert_eq!(spec.items.len(), 1);
        if let Item::Project(proj) = &spec.items[0] {
            assert_eq!(proj.name, "my-bot");
            assert_eq!(proj.version, "0.1.0");
            assert_eq!(proj.language, "rust");
        } else {
            panic!("expected Project");
        }
    }

    #[test]
    fn test_parse_spec_khold() {
        let input = r#"
project "my-trading-bot" { version = "0.1.0", language = "rust" }

type Price = f64
type OHLCV = struct { open: Price, high: Price }

enum Regime { Trending, Ranging, Volatile }

interface MarketDataFeed {
    fn fetch(symbol: String, from: u64) -> Result<Vec<OHLCV>, String>
}

fn classify(data: Vec<OHLCV>) -> Regime {
    constraint { time: O(n) space: O(1) pure: true }
    test trending: { assert classify(data) == Trending }
}
"#;
        let mut p = Parser::new(input);
        let spec = p.parse_spec().unwrap();
        assert_eq!(spec.items.len(), 6);
    }
}
