mod params;
mod source;
mod types;

use crate::ast::*;
use crate::error::{CompileError, Result};
use params::parse_params_v2;
use source::Source;
use std::path::Path;
use types::*;

pub fn parse_file(path: &Path) -> Result<Program> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CompileError::at(path, 0, format!("unable to read file: {e}")))?;
    parse_source(path, &text)
}

pub fn parse_source(path: &Path, text: &str) -> Result<Program> {
    let mut src = Source::new(path, text);
    let body = parse_block(&mut src, -1)?;
    Ok(Program { body })
}

fn parse_block(src: &mut Source, parent_indent: i32) -> Result<Block> {
    let mut stmts = Vec::new();
    while src.peek()? {
        if src.indent <= parent_indent {
            break;
        }
        stmts.push(parse_statement(src)?);
    }
    Ok(stmts)
}

fn parse_statement(src: &mut Source) -> Result<Stmt> {
    let line = src.line_number();
    let text = src.text.clone();
    let path = src.path.clone();

    if text == "fi" || text == "esac" {
        return Err(CompileError::at(
            &path,
            line,
            format!("'{text}' is not valid in scriptopotamus"),
        ));
    }
    if text == "elif"
        || text.starts_with("elif ")
        || text == "else"
        || text.starts_with("else ")
        || text.starts_with("else;")
    {
        return Err(CompileError::at(
            &path,
            line,
            format!(
                "'{}' without matching 'if'",
                text.split_whitespace().next().unwrap_or(&text)
            ),
        ));
    }

    // Inline control on one line: if $c; -> a; else -> b
    if text.contains(';') && !text.starts_with("case ") && !looks_like_function(&text) {
        return parse_inline_line(src, line, &text);
    }

    if let Some(rest) = text.strip_prefix("if ") {
        return parse_if(src, rest.to_string());
    }
    if let Some(caps) = regex::Regex::new(r"^for\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\s+(.+)$")
        .unwrap()
        .captures(&text)
    {
        return parse_for(src, caps[1].to_string(), caps[2].to_string());
    }
    if let Some(rest) = text.strip_prefix("while ") {
        return parse_while(src, rest.to_string());
    }
    if let Some(rest) = text.strip_prefix("case ") {
        return parse_case(src, rest.to_string());
    }
    if let Some(rest) = text.strip_prefix("->") {
        src.next();
        return Ok(Stmt::Echo {
            value: parse_value(rest.trim()),
            line,
        });
    }
    if text.starts_with('@') {
        return parse_macro(src, &text);
    }
    if looks_like_function(&text) {
        return parse_function(src);
    }
    if let Some(stmt) = try_assignment(line, &path, &text)? {
        src.next();
        return Ok(stmt);
    }
    if let Some(stmt) = try_declaration(line, &path, &text)? {
        src.next();
        return Ok(stmt);
    }
    parse_bash(src)
}

fn parse_inline_line(src: &mut Source, line: usize, text: &str) -> Result<Stmt> {
    let indent = src.indent;
    src.next();
    let parts = split_semicolons(text);
    let mut stmts = Vec::new();
    for part in &parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        stmts.push(parse_fragment(src, line, part)?);
    }
    assemble_inline_if(src, indent, line, stmts)
}

fn parse_fragment(src: &mut Source, line: usize, text: &str) -> Result<Stmt> {
    let path = src.path.clone();
    if let Some(rest) = text.strip_prefix("->") {
        return Ok(Stmt::Echo {
            value: parse_value(rest.trim()),
            line,
        });
    }
    if let Some(rest) = text.strip_prefix("if ") {
        return Ok(Stmt::If {
            branches: vec![IfBranch {
                condition: parse_condition(rest.trim()),
                body: Vec::new(),
            }],
            else_body: None,
            line,
        });
    }
    if let Some(rest) = text.strip_prefix("elif ") {
        return Ok(Stmt::If {
            branches: vec![IfBranch {
                condition: parse_condition(rest.trim()),
                body: Vec::new(),
            }],
            else_body: None,
            line,
        });
    }
    if text == "else" {
        return Ok(Stmt::If {
            branches: Vec::new(),
            else_body: Some(Vec::new()),
            line,
        });
    }
    if let Some(rest) = text.strip_prefix("else ") {
        return Ok(Stmt::If {
            branches: Vec::new(),
            else_body: Some(vec![parse_fragment(src, line, rest)?]),
            line,
        });
    }
    if let Some(rest) = text.strip_prefix("while ") {
        return Ok(Stmt::While {
            condition: parse_condition(rest.trim()),
            body: Vec::new(),
            line,
        });
    }
    if let Some(caps) = regex::Regex::new(r"^for\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\s+(.+)$")
        .unwrap()
        .captures(text)
    {
        return Ok(Stmt::For {
            var: caps[1].to_string(),
            items: split_words(&caps[2]),
            body: Vec::new(),
            line,
        });
    }
    if let Some(stmt) = try_assignment(line, &path, text)? {
        return Ok(stmt);
    }
    if let Some(stmt) = try_declaration(line, &path, text)? {
        return Ok(stmt);
    }
    if text.starts_with('@') {
        return parse_macro_text(line, text);
    }
    Ok(Stmt::Bash {
        raw: text.to_string(),
        line,
    })
}

fn assemble_inline_if(
    src: &mut Source,
    indent: i32,
    line: usize,
    stmts: Vec<Stmt>,
) -> Result<Stmt> {
    if stmts.is_empty() {
        return Ok(Stmt::Bash {
            raw: String::new(),
            line,
        });
    }

    // Pattern: If(cond, []) then body stmts then maybe elif/else markers
    let mut iter = stmts.into_iter().peekable();
    let first = iter.next().unwrap();

    match first {
        Stmt::If { mut branches, .. } if !branches.is_empty() => {
            let mut body = Vec::new();
            while let Some(peek) = iter.peek() {
                match peek {
                    Stmt::If {
                        branches: b,
                        else_body: e,
                        ..
                    } if (b.len() == 1 && e.is_none()) || (b.is_empty() && e.is_some()) => {
                        break;
                    }
                    _ => body.push(iter.next().unwrap()),
                }
            }
            branches[0].body = body;

            while let Some(next) = iter.next() {
                match next {
                    Stmt::If {
                        branches: b,
                        else_body: None,
                        ..
                    } if b.len() == 1 => {
                        let mut branch = b.into_iter().next().unwrap();
                        let mut body = Vec::new();
                        while let Some(peek) = iter.peek() {
                            match peek {
                                Stmt::If {
                                    branches: bb,
                                    else_body: e,
                                    ..
                                } if (bb.len() == 1 && e.is_none())
                                    || (bb.is_empty() && e.is_some()) =>
                                {
                                    break;
                                }
                                _ => body.push(iter.next().unwrap()),
                            }
                        }
                        branch.body = body;
                        branches.push(branch);
                    }
                    Stmt::If {
                        branches: b,
                        else_body: Some(mut eb),
                        ..
                    } if b.is_empty() => {
                        eb.extend(iter);
                        let else_body = Some(eb);
                        return finish_if(src, indent, line, branches, else_body);
                    }
                    other => {
                        if let Some(br) = branches.last_mut() {
                            br.body.push(other);
                        }
                    }
                }
            }
            finish_if(src, indent, line, branches, None)
        }
        Stmt::While {
            condition,
            mut body,
            line,
        } => {
            body.extend(iter);
            Ok(Stmt::While {
                condition,
                body,
                line,
            })
        }
        Stmt::For {
            var,
            items,
            mut body,
            line,
        } => {
            body.extend(iter);
            Ok(Stmt::For {
                var,
                items,
                body,
                line,
            })
        }
        other => {
            if iter.peek().is_none() {
                Ok(other)
            } else {
                let mut raw = format!("{other:?}");
                for s in iter {
                    raw.push(';');
                    raw.push_str(&format!("{s:?}"));
                }
                Ok(Stmt::Bash { raw, line })
            }
        }
    }
}

fn finish_if(
    src: &mut Source,
    indent: i32,
    line: usize,
    mut branches: Vec<IfBranch>,
    mut else_body: Option<Block>,
) -> Result<Stmt> {
    while else_body.is_none() && src.peek()? && src.indent == indent {
        let t = src.text.clone();
        if let Some(rest) = t.strip_prefix("elif ") {
            let eline = src.line_number();
            let rest = rest.to_string();
            src.next();
            let (cond, inline) = split_first_semi(&rest);
            let body = if let Some(inline) = inline {
                let mut b = Vec::new();
                for part in split_semicolons(inline) {
                    let part = part.trim();
                    if !part.is_empty() {
                        b.push(parse_fragment(src, eline, part)?);
                    }
                }
                b
            } else {
                parse_block(src, indent)?
            };
            branches.push(IfBranch {
                condition: parse_condition(cond.trim()),
                body,
            });
        } else if t == "else" || t.starts_with("else ") || t.starts_with("else;") {
            let eline = src.line_number();
            let rest = t
                .strip_prefix("else")
                .unwrap()
                .trim_start_matches(';')
                .trim()
                .to_string();
            src.next();
            else_body = Some(if !rest.is_empty() {
                let mut b = Vec::new();
                for part in split_semicolons(&rest) {
                    let part = part.trim();
                    if !part.is_empty() {
                        b.push(parse_fragment(src, eline, part)?);
                    }
                }
                b
            } else {
                parse_block(src, indent)?
            });
            break;
        } else {
            break;
        }
    }
    Ok(Stmt::If {
        branches,
        else_body,
        line,
    })
}

fn parse_if(src: &mut Source, condition: String) -> Result<Stmt> {
    let indent = src.indent;
    let line = src.line_number();
    let (cond, inline) = split_first_semi(&condition);
    src.next();
    let body = if let Some(inline) = inline {
        let mut b = Vec::new();
        for part in split_semicolons(inline) {
            let part = part.trim();
            if !part.is_empty() {
                b.push(parse_fragment(src, line, part)?);
            }
        }
        b
    } else {
        parse_block(src, indent)?
    };
    let branches = vec![IfBranch {
        condition: parse_condition(cond.trim()),
        body,
    }];
    finish_if(src, indent, line, branches, None)
}

fn parse_for(src: &mut Source, var: String, items_raw: String) -> Result<Stmt> {
    let indent = src.indent;
    let line = src.line_number();
    let (items_part, inline) = split_first_semi(&items_raw);
    let items = split_words(items_part.trim());
    src.next();
    let body = if let Some(inline) = inline {
        let mut b = Vec::new();
        for part in split_semicolons(inline) {
            let part = part.trim();
            if !part.is_empty() {
                b.push(parse_fragment(src, line, part)?);
            }
        }
        b
    } else {
        parse_block(src, indent)?
    };
    Ok(Stmt::For {
        var,
        items,
        body,
        line,
    })
}

fn parse_while(src: &mut Source, condition: String) -> Result<Stmt> {
    let indent = src.indent;
    let line = src.line_number();
    let (cond, inline) = split_first_semi(&condition);
    src.next();
    let body = if let Some(inline) = inline {
        let mut b = Vec::new();
        for part in split_semicolons(inline) {
            let part = part.trim();
            if !part.is_empty() {
                b.push(parse_fragment(src, line, part)?);
            }
        }
        b
    } else {
        parse_block(src, indent)?
    };
    Ok(Stmt::While {
        condition: parse_condition(cond.trim()),
        body,
        line,
    })
}

fn parse_case(src: &mut Source, subject_raw: String) -> Result<Stmt> {
    let indent = src.indent;
    let line = src.line_number();
    let mut subject = subject_raw.trim().to_string();
    if let Some(s) = subject.strip_suffix(" in") {
        subject = s.trim().to_string();
    }
    src.next();
    let mut arms = Vec::new();
    while src.peek()? && src.indent > indent {
        arms.push(parse_case_arm(src)?);
    }
    Ok(Stmt::Case {
        subject,
        arms,
        line,
    })
}

fn parse_case_arm(src: &mut Source) -> Result<CaseArm> {
    let indent = src.indent;
    let line = src.line_number();
    let text = src.text.clone();
    let path = src.path.clone();

    if CaseTerminator::parse(&text).is_some() {
        return Err(CompileError::at(
            &path,
            line,
            format!("unexpected '{text}': end the arm's line with it or indent it under the arm"),
        ));
    }

    let mut pattern = String::new();
    let mut depth = 0i32;
    let mut quote = None::<char>;
    let mut end = None;
    for (i, c) in text.char_indices() {
        if let Some(q) = quote {
            pattern.push(c);
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
            pattern.push(c);
        } else if c == '(' {
            depth += 1;
            pattern.push(c);
        } else if c == ')' && depth == 0 {
            end = Some(i);
            break;
        } else if c == ')' {
            depth -= 1;
            pattern.push(c);
        } else {
            pattern.push(c);
        }
    }
    let end = end.ok_or_else(|| {
        CompileError::at(
            &path,
            line,
            format!("expected 'pattern)' in case, got '{text}'"),
        )
    })?;
    if pattern.is_empty() {
        return Err(CompileError::at(
            &path,
            line,
            format!("expected 'pattern)' in case, got '{text}'"),
        ));
    }

    let rest = text[end + 1..].trim();
    let (rest, mut terminator) = strip_terminator(rest);
    let mut body = Vec::new();
    if !rest.is_empty() {
        for part in split_semicolons(rest) {
            let part = part.trim();
            if !part.is_empty() {
                body.push(parse_fragment(src, line, part)?);
            }
        }
        src.next();
    } else {
        src.next();
    }
    body.extend(parse_block(src, indent)?);

    if let Some(Stmt::Bash { raw, .. }) = body.last() {
        if let Some(t) = CaseTerminator::parse(raw.trim()) {
            terminator = t;
            body.pop();
        }
    }

    Ok(CaseArm {
        pattern,
        body,
        terminator,
    })
}

fn parse_macro(src: &mut Source, text: &str) -> Result<Stmt> {
    let line = src.line_number();
    let indent = src.indent;
    // @name(...) definition or @name call
    let re = regex::Regex::new(r"^@([A-Za-z_][A-Za-z0-9_]*)(\((.*)\))?(\s+(.*))?$").unwrap();
    let caps = re
        .captures(text)
        .ok_or_else(|| src.error(format!("invalid macro '{text}'")))?;
    let name = caps[1].to_string();

    if caps.get(2).is_some() && caps.get(5).is_none() {
        // Could be definition @name() or @name(args) call with empty body line
        let params_raw = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        src.next();
        // If next lines are indented, it's a definition
        if src.peek()? && src.indent > indent {
            let body = parse_block(src, indent)?;
            let params = if params_raw.trim().is_empty() {
                Vec::new()
            } else {
                split_words(params_raw)
            };
            return Ok(Stmt::MacroDef {
                name,
                params,
                body,
                line,
            });
        }
        // Call with ()
        return Ok(Stmt::MacroCall {
            name,
            args: Vec::new(),
            line,
        });
    }

    src.next();
    if name != "import" && src.peek()? && src.indent > indent && caps.get(5).is_none() {
        let body = parse_block(src, indent)?;
        return Ok(Stmt::MacroDef {
            name,
            params: Vec::new(),
            body,
            line,
        });
    }

    let args = if let Some(m) = caps.get(5) {
        split_words(m.as_str())
    } else if let Some(m) = caps.get(3) {
        split_words(m.as_str())
    } else {
        Vec::new()
    };
    Ok(Stmt::MacroCall { name, args, line })
}

fn parse_macro_text(line: usize, text: &str) -> Result<Stmt> {
    let re = regex::Regex::new(r"^@([A-Za-z_][A-Za-z0-9_]*)(\((.*)\))?(\s+(.*))?$").unwrap();
    let caps = re
        .captures(text)
        .ok_or_else(|| CompileError::new("<inline>", line, format!("invalid macro '{text}'")))?;
    let name = caps[1].to_string();
    let args = if let Some(m) = caps.get(5) {
        split_words(m.as_str())
    } else if let Some(m) = caps.get(3) {
        split_words(m.as_str())
    } else {
        Vec::new()
    };
    Ok(Stmt::MacroCall { name, args, line })
}

fn parse_function(src: &mut Source) -> Result<Stmt> {
    let indent = src.indent;
    let line = src.line_number();
    let path = src.path.clone();
    let text = src.text.clone();

    let mut is_command = false;
    let mut name = text.split('(').next().unwrap().to_string();
    if name.starts_with('.') {
        is_command = true;
        name = name[1..].to_string();
    }

    let mut docs: Vec<(String, String)> = src
        .usage_comments()
        .into_iter()
        .map(|s| (String::new(), s))
        .collect();
    let mut pending_docs = Vec::new();
    let mut params_raw = String::new();
    let mut depth = 0i32;
    let mut started = false;
    let mut tail = String::new();
    let mut collecting = true;

    loop {
        let cur = if collecting {
            src.text.clone()
        } else {
            break;
        };
        let mut comment_stripped = cur.clone();
        let preceding = if src.line_number() != line {
            src.usage_comments()
        } else {
            Vec::new()
        };
        pending_docs.extend(preceding);
        if let Some(idx) = find_comment(&cur) {
            if let Some(comment) = cur[idx..].strip_prefix("#?") {
                pending_docs.push(comment.trim().to_string());
            }
        }
        if !pending_docs.is_empty() {
            let key = if src.line_number() == line || cur.trim_start().starts_with(')') {
                String::new()
            } else {
                cur.trim_start()
                    .trim_start_matches('[')
                    .trim_start()
                    .split(|c: char| c.is_whitespace() || "<>] /|".contains(c))
                    .next()
                    .unwrap_or("")
                    .to_string()
            };
            for comment in pending_docs.drain(..) {
                docs.push((key.clone(), comment));
            }
        }
        if let Some(idx) = find_comment(&comment_stripped) {
            comment_stripped = comment_stripped[..idx].trim_end().to_string();
        }
        for c in comment_stripped.chars() {
            if c == '(' {
                depth += 1;
                started = true;
                if depth > 1 {
                    params_raw.push(c);
                }
                continue;
            }
            if c == ')' {
                depth -= 1;
                if depth == 0 {
                    collecting = false;
                    continue;
                }
                params_raw.push(c);
                continue;
            }
            if started && depth > 0 {
                params_raw.push(c);
            } else if started && depth == 0 {
                tail.push(c);
            }
        }
        if depth == 0 && started {
            src.next();
            break;
        }
        params_raw.push(' ');
        src.next();
        if !src.peek()? {
            return Err(CompileError::at(
                &path,
                line,
                format!("unclosed '(' in {name}"),
            ));
        }
    }

    let tail = tail.trim().to_string();
    let mut body = Vec::new();
    if !tail.is_empty() {
        if let Some(rest) = tail.strip_prefix("->") {
            body.push(Stmt::Echo {
                value: parse_value(rest.trim()),
                line,
            });
        } else {
            return Err(CompileError::at(
                &path,
                line,
                format!("unexpected '{tail}' after {name}(...)"),
            ));
        }
    }

    body.extend(parse_block(src, indent)?);
    let params = parse_params_v2(&path, line, &params_raw)?;

    Ok(Stmt::Function {
        name,
        is_command,
        docs,
        params,
        body,
        line,
    })
}

fn find_comment(s: &str) -> Option<usize> {
    let mut quote = None::<char>;
    for (i, c) in s.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c == '#' {
            return Some(i);
        }
    }
    None
}

fn try_assignment(line: usize, path: &Path, text: &str) -> Result<Option<Stmt>> {
    let typed = regex::Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)<([^>]*)>\s*=\s*(.*)$").unwrap();
    if let Some(caps) = typed.captures(text) {
        let name = caps[1].to_string();
        let ty = parse_type(path, line, &caps[2])?;
        let value = parse_value(caps[3].trim());
        return Ok(Some(Stmt::Assignment {
            name,
            ty: Some(ty),
            value,
            line,
        }));
    }
    let plain = regex::Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$").unwrap();
    if let Some(caps) = plain.captures(text) {
        // Avoid matching flags like --opt=val as assignment at statement level if starts with -
        return Ok(Some(Stmt::Assignment {
            name: caps[1].to_string(),
            ty: None,
            value: parse_value(caps[2].trim()),
            line,
        }));
    }
    Ok(None)
}

fn try_declaration(line: usize, path: &Path, text: &str) -> Result<Option<Stmt>> {
    let re = regex::Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)<([^>]*)>$").unwrap();
    if let Some(caps) = re.captures(text) {
        let name = caps[1].to_string();
        let ty = parse_type(path, line, &caps[2])?;
        return Ok(Some(Stmt::Declaration { name, ty, line }));
    }
    Ok(None)
}

fn parse_bash(src: &mut Source) -> Result<Stmt> {
    let indent = src.indent;
    let line = src.line_number();
    let mut lines = vec![src.text.clone()];
    src.next();
    while src.peek()? && src.indent > indent {
        let raw = src.current_raw_line().unwrap_or("");
        let content = if raw.len() > indent as usize {
            raw[indent as usize..].to_string()
        } else {
            src.text.clone()
        };
        lines.push(content);
        src.next();
    }
    Ok(Stmt::Bash {
        raw: lines.join("\n"),
        line,
    })
}
