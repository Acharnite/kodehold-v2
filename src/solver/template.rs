use std::collections::HashMap;

use crate::solver::pattern_db::Pattern;

pub fn apply(pattern: &Pattern, variables: &HashMap<String, String>) -> String {
    let mut code = pattern.template.clone();
    for (key, value) in variables {
        code = code.replace(&format!("{{{{{key}}}}}"), value);
    }
    code
}

pub fn extract_variables(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // skip second {
            let mut name = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                name.push(ch);
            }
            vars.push(name);
        }
    }
    vars
}
