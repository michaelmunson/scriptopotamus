use super::types::parse_type;
use crate::ast::{Param, Type, TypeKind};
use crate::error::{CompileError, Result};
use std::path::Path;

fn tokenize_params(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut in_type = false;
    let mut brace_depth = 0i32;
    for c in input.chars() {
        if in_type {
            word.push(c);
            if c == '{' || c == '(' {
                brace_depth += 1;
            } else if c == '}' || c == ')' {
                brace_depth -= 1;
            } else if c == '>' && brace_depth == 0 {
                in_type = false;
            }
            continue;
        }
        match c {
            '<' => {
                word.push(c);
                in_type = true;
                brace_depth = 0;
            }
            '[' => {
                if !word.is_empty() {
                    tokens.push(std::mem::take(&mut word));
                }
                tokens.push("[".to_string());
            }
            ']' => {
                if !word.is_empty() {
                    tokens.push(std::mem::take(&mut word));
                }
                tokens.push("]".to_string());
            }
            '|' | '/' => {
                if !word.is_empty() {
                    tokens.push(std::mem::take(&mut word));
                }
                tokens.push(c.to_string());
            }
            c if c.is_whitespace() => {
                if !word.is_empty() {
                    tokens.push(std::mem::take(&mut word));
                }
            }
            '#' => break,
            _ => word.push(c),
        }
    }
    if !word.is_empty() {
        tokens.push(word);
    }
    tokens
}

fn parse_one(
    path: &Path,
    line: usize,
    tokens: &[String],
    i: usize,
    optional_context: bool,
) -> Result<(Param, usize)> {
    if tokens[i] == "[" {
        return parse_group(path, line, tokens, i, true);
    }
    parse_leaf(path, line, &tokens[i], optional_context).map(|p| (p, i + 1))
}

fn parse_group(
    path: &Path,
    line: usize,
    tokens: &[String],
    start: usize,
    optional: bool,
) -> Result<(Param, usize)> {
    let mut i = start + 1;
    let mut inner = Vec::new();
    let mut depth = 1;
    while i < tokens.len() {
        if tokens[i] == "[" {
            depth += 1;
            inner.push(tokens[i].clone());
            i += 1;
        } else if tokens[i] == "]" {
            depth -= 1;
            if depth == 0 {
                i += 1;
                break;
            }
            inner.push(tokens[i].clone());
            i += 1;
        } else {
            inner.push(tokens[i].clone());
            i += 1;
        }
    }
    if depth != 0 {
        return Err(CompileError::at(path, line, "unclosed '[' in parameters"));
    }

    // Check exclusive inside group
    let params = parse_param_list_with_exclusive(path, line, &inner, true)?;
    if params.len() == 1 {
        let mut p = params.into_iter().next().unwrap();
        match &mut p {
            Param::Arg { optional: o, .. } | Param::Opt { optional: o, .. } => *o = true,
            Param::Group { optional: o, .. } => *o = true,
            _ => {}
        }
        return Ok((p, i));
    }
    Ok((Param::Group { optional, params }, i))
}

fn parse_param_list_with_exclusive(
    path: &Path,
    line: usize,
    tokens: &[String],
    optional_context: bool,
) -> Result<Vec<Param>> {
    let tokens = merge_flag_aliases(tokens);
    let mut params = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "|" {
            return Err(CompileError::at(path, line, "unexpected '|' in parameters"));
        }
        let (param, next) = parse_one(path, line, tokens.as_slice(), i, optional_context)?;
        i = next;
        if i < tokens.len() && tokens[i] == "|" {
            let mut options = vec![param];
            while i < tokens.len() && tokens[i] == "|" {
                i += 1;
                if i >= tokens.len() || tokens[i] == "|" {
                    return Err(CompileError::at(
                        path,
                        line,
                        "expected a parameter after '|'",
                    ));
                }
                let (next_param, next_i) =
                    parse_one(path, line, tokens.as_slice(), i, optional_context)?;
                options.push(next_param);
                i = next_i;
            }
            params.push(Param::Exclusive { options });
        } else {
            params.push(param);
        }
    }
    Ok(params)
}

/// Merge `-s / --select` into a single token `-s|--select` when both sides are flags.
fn merge_flag_aliases(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if i + 2 < tokens.len()
            && tokens[i + 1] == "/"
            && tokens[i].starts_with('-')
            && !tokens[i].starts_with("[")
            && tokens[i + 2].starts_with('-')
            && tokens[i + 2] != "/"
        {
            let mut aliases = vec![tokens[i].clone()];
            i += 2;
            while i < tokens.len() && tokens[i].starts_with('-') {
                // strip type/default from first, keep on last? keep raw tokens joined
                let part = tokens[i].clone();
                // If this token has <type>, attach to combined
                aliases.push(part);
                if i + 1 < tokens.len()
                    && tokens[i + 1] == "/"
                    && i + 2 < tokens.len()
                    && tokens[i + 2].starts_with('-')
                {
                    i += 2;
                    continue;
                }
                break;
            }
            // Combine: first flag names without types + type from last if any
            let mut names = Vec::new();
            let mut suffix = String::new();
            for a in &aliases {
                if let Some(pos) = a.find('<') {
                    names.push(a[..pos].to_string());
                    suffix = a[pos..].to_string();
                } else if let Some((n, d)) = a.split_once('=') {
                    names.push(n.to_string());
                    suffix = format!("={d}");
                } else {
                    names.push(a.clone());
                }
            }
            out.push(format!("{}{}", names.join("|"), suffix));
            i += 1;
            continue;
        }
        out.push(tokens[i].clone());
        i += 1;
    }
    out
}

fn parse_leaf(path: &Path, line: usize, token: &str, optional_context: bool) -> Result<Param> {
    let token = token.trim();
    if matches!(token, "/" | "|" | "]") {
        return Err(CompileError::at(
            path,
            line,
            format!("unexpected '{token}' in parameters"),
        ));
    }
    if token.is_empty() {
        return Err(CompileError::at(path, line, "empty parameter"));
    }

    // Rest: *name
    if let Some(name) = token.strip_prefix('*') {
        let (name, ty, _) = parse_name_type_default(path, line, name, false)?;
        return Ok(Param::Rest { name, ty });
    }

    let (token, repeat) = if let Some(base) = token.strip_suffix("...") {
        (base, true)
    } else {
        (token, false)
    };

    let is_flag = token.starts_with('-');
    let (name, ty, default) = parse_name_type_default(path, line, token, is_flag)?;

    if is_flag {
        let flags = split_flag_aliases(&name);
        let var_name = flag_to_name(&flags[0]);
        let ty = if !token.contains('<') && default.is_none() {
            Type::plain(TypeKind::Bool)
        } else {
            ty
        };
        Ok(Param::Opt {
            name: var_name,
            flags,
            ty,
            optional: optional_context,
            default,
            repeat,
        })
    } else {
        let ty = if !token.contains('<') {
            Type::plain(TypeKind::Str)
        } else {
            ty
        };
        Ok(Param::Arg {
            name,
            ty,
            optional: optional_context || repeat,
            default,
            repeat,
        })
    }
}

fn parse_name_type_default(
    path: &Path,
    line: usize,
    token: &str,
    _is_flag: bool,
) -> Result<(String, Type, Option<String>)> {
    let mut name = token.to_string();
    let mut ty = Type::plain(TypeKind::Any);
    let mut default = None;

    if let Some(start) = name.find('<') {
        let end = name
            .rfind('>')
            .ok_or_else(|| CompileError::at(path, line, format!("unclosed '<' in '{token}'")))?;
        let type_str = name[start + 1..end].to_string();
        let after = name[end + 1..].to_string();
        name = name[..start].to_string();

        if let Some((t, d)) = split_type_default(&type_str) {
            ty = parse_type(path, line, t)?;
            default = Some(unquote(d));
        } else {
            ty = parse_type(path, line, &type_str)?;
        }
        if let Some(d) = after.strip_prefix('=') {
            default = Some(unquote(d));
        }
    } else if let Some((n, d)) = name.split_once('=') {
        let d = d.to_string();
        name = n.to_string();
        default = Some(unquote(&d));
    }

    // Handle --flag... repeat
    let mut repeat = false;
    if name.ends_with("...") {
        name = name.trim_end_matches("...").to_string();
        repeat = true;
    }
    let _ = repeat;

    Ok((name, ty, default))
}

fn split_type_default(s: &str) -> Option<(&str, &str)> {
    // Find = outside braces
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '{' | '(' => depth += 1,
            '}' | ')' => depth -= 1,
            '=' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
            _ => {}
        }
    }
    None
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn split_flag_aliases(name: &str) -> Vec<String> {
    name.split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn flag_to_name(flag: &str) -> String {
    flag.trim_start_matches('-').replace('-', "_")
}

pub fn parse_params_v2(path: &Path, line: usize, input: &str) -> Result<Vec<Param>> {
    let tokens = tokenize_params(input);
    parse_param_list_with_exclusive(path, line, &tokens, false)
}
