use crate::ast::{Type, TypeKind};
use crate::error::{CompileError, Result};
use std::path::Path;

pub fn parse_type(path: &Path, line: usize, raw: &str) -> Result<Type> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(CompileError::at(path, line, "empty type"));
    }

    let (base_part, literals) = if let Some(start) = raw.find('{') {
        let end = raw
            .rfind('}')
            .ok_or_else(|| CompileError::at(path, line, format!("unclosed '{{' in type '{raw}'")))?;
        let inside = &raw[start + 1..end];
        let lits: Vec<String> = split_words(inside);
        (raw[..start].trim(), Some(lits))
    } else if let Some(start) = raw.find('(') {
        let end = raw
            .rfind(')')
            .ok_or_else(|| CompileError::at(path, line, format!("unclosed '(' in type '{raw}'")))?;
        let inside = &raw[start + 1..end];
        let lits: Vec<String> = split_words(inside);
        (raw[..start].trim(), Some(lits))
    } else {
        (raw, None)
    };

    let mut is_const = false;
    let mut kind = None;
    for word in base_part.split_whitespace() {
        match word {
            "const" => is_const = true,
            "str" | "string" => kind = Some(TypeKind::Str),
            "int" | "integer" => kind = Some(TypeKind::Int),
            "float" => kind = Some(TypeKind::Float),
            "bool" | "boolean" => kind = Some(TypeKind::Bool),
            "list" | "array" => kind = Some(TypeKind::List),
            "dict" => kind = Some(TypeKind::Dict),
            "any" => kind = Some(TypeKind::Any),
            other => {
                return Err(CompileError::at(
                    path,
                    line,
                    format!("unknown type '{other}'"),
                ));
            }
        }
    }

    let kind = match kind {
        Some(k) => k,
        None if is_const => TypeKind::Any,
        None => {
            return Err(CompileError::at(
                path,
                line,
                format!("missing type kind in '{raw}'"),
            ));
        }
    };

    Ok(Type {
        kind,
        is_const,
        literals,
    })
}

pub fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None::<char>;
    for c in input.chars() {
        if let Some(q) = quote {
            word.push(c);
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            word.push(c);
            quote = Some(c);
        } else if c.is_whitespace() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

pub fn parse_value(raw: &str) -> crate::ast::Value {
    let raw = raw.trim();
    if let Some(inner) = raw.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        let words = split_words(inner);
        if !words.is_empty() && words.iter().all(|w| w.contains('=')) {
            let entries = words
                .into_iter()
                .map(|w| {
                    let (k, v) = w.split_once('=').unwrap();
                    (k.to_string(), v.to_string())
                })
                .collect();
            return crate::ast::Value::Dict(entries);
        }
        return crate::ast::Value::List(words);
    }
    let math_re = regex::Regex::new(r#"^[^"'()]+[\s]+[-+*/%][\s]+[^"'()]+$"#).unwrap();
    if math_re.is_match(raw) {
        return crate::ast::Value::Math(raw.to_string());
    }
    crate::ast::Value::Scalar(raw.to_string())
}

pub fn parse_condition(raw: &str) -> crate::ast::Condition {
    let mut raw = raw.trim().to_string();
    let mut negated = false;
    if let Some(rest) = raw.strip_prefix("! ") {
        negated = true;
        raw = rest.trim().to_string();
    }
    let bool_re =
        regex::Regex::new(r"^\$(\{[A-Za-z_][A-Za-z0-9_]*\}|--?[A-Za-z_][A-Za-z0-9_-]*|[A-Za-z_][A-Za-z0-9_]*)$")
            .unwrap();
    let kind = if bool_re.is_match(&raw) {
        crate::ast::ConditionKind::Bool
    } else if !raw.is_empty()
        && !matches!(
            raw.chars().next(),
            Some('$' | '"' | '\'' | '0'..='9' | '-')
        )
    {
        crate::ast::ConditionKind::Command
    } else {
        crate::ast::ConditionKind::Test
    };
    crate::ast::Condition {
        kind,
        raw,
        negated,
    }
}

pub fn split_semicolons(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut quote = None::<char>;
    for c in text.chars() {
        if let Some(q) = quote {
            cur.push(c);
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            cur.push(c);
            quote = Some(c);
        } else if c == ';' {
            parts.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() || parts.is_empty() {
        parts.push(cur);
    }
    parts
}

pub fn split_first_semi(text: &str) -> (&str, Option<&str>) {
    let mut quote = None::<char>;
    for (i, c) in text.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c == ';' {
            let after = text[i + 1..].trim();
            return (
                text[..i].trim(),
                if after.is_empty() { None } else { Some(after) },
            );
        }
    }
    (text.trim(), None)
}

pub fn strip_terminator(rest: &str) -> (&str, crate::ast::CaseTerminator) {
    use crate::ast::CaseTerminator;
    let rest = rest.trim();
    for term in [";;&", ";;", ";&"] {
        if let Some(before) = rest.strip_suffix(term) {
            let before = before.trim();
            return (
                before,
                CaseTerminator::parse(term).unwrap_or(CaseTerminator::Break),
            );
        }
    }
    (rest, CaseTerminator::Break)
}

pub fn looks_like_function(text: &str) -> bool {
    let re = regex::Regex::new(r"^(\.|\.?[A-Za-z_][A-Za-z0-9_-]*)\(").unwrap();
    re.is_match(text)
}
