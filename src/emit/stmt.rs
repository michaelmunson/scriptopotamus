use super::params::{emit_function_params, flatten_leaves};
use super::{emit_condition, emit_value, type_test, Emitter, RootKind};
use crate::ast::*;
use crate::error::{CompileError, Result};

pub fn emit_stmt(em: &mut Emitter, stmt: &Stmt) -> Result<()> {
    match stmt {
        Stmt::Declaration { name, ty, line } => {
            emit_declare(em, name, ty, None, *line)?;
        }
        Stmt::Assignment {
            name,
            ty,
            value,
            line,
        } => {
            if let Some(ty) = ty {
                emit_declare(em, name, ty, Some(value), *line)?;
            } else {
                let val = emit_value(value);
                if let Some(existing) = em.types.get(name) {
                    if existing.is_const {
                        return Err(CompileError::new(
                            "<input>",
                            *line,
                            format!("cannot reassign const '{name}'"),
                        ));
                    }
                }
                match value {
                    Value::List(_) => em.emit_line(&format!("{name}={val}")),
                    Value::Dict(_) => em.emit_line(&format!("{name}={val}")),
                    _ => em.emit_line(&format!("{name}={val}")),
                }
                if let Some(ty) = em.types.get(name).cloned() {
                    if let Some(test) = type_test(&format!("${name}"), &ty) {
                        em.emit_line(&format!("if [[ {test} ]]; then"));
                        em.emit_line(&format!(
                            "  echo \"{name}: expected {}, received ${name}\" >&2",
                            ty.description()
                        ));
                        em.emit_line("  exit 1");
                        em.emit_line("fi");
                    }
                }
            }
        }
        Stmt::If {
            branches,
            else_body,
            ..
        } => {
            for (i, branch) in branches.iter().enumerate() {
                let kw = if i == 0 { "if" } else { "elif" };
                em.emit_line(&format!("{kw} {}; then", emit_condition(&branch.condition)));
                em.indent();
                emit_body(em, &branch.body)?;
                em.dedent();
            }
            if let Some(body) = else_body {
                em.emit_line("else");
                em.indent();
                emit_body(em, body)?;
                em.dedent();
            }
            em.emit_line("fi");
        }
        Stmt::For {
            var, items, body, ..
        } => {
            let items_str = items
                .iter()
                .map(|i| expand_for_item(i))
                .collect::<Vec<_>>()
                .join(" ");
            em.emit_line(&format!("for {var} in {items_str}; do"));
            em.indent();
            emit_body(em, body)?;
            em.dedent();
            em.emit_line("done");
        }
        Stmt::While {
            condition, body, ..
        } => {
            em.emit_line(&format!("while {}; do", emit_condition(condition)));
            em.indent();
            emit_body(em, body)?;
            em.dedent();
            em.emit_line("done");
        }
        Stmt::Case {
            subject, arms, ..
        } => {
            em.emit_line(&format!("case {} in", super::rewrite_refs(subject)));
            em.indent();
            for arm in arms {
                em.emit_line(&format!("{})", arm.pattern));
                em.indent();
                emit_body(em, &arm.body)?;
                em.emit_line(arm.terminator.as_bash());
                em.dedent();
            }
            em.dedent();
            em.emit_line("esac");
        }
        Stmt::Echo { value, .. } => {
            let v = emit_value(value);
            if v.is_empty() {
                em.emit_line("echo");
            } else {
                em.emit_line(&format!("echo {v}"));
            }
        }
        Stmt::Function {
            name,
            is_command,
            params,
            body,
            line,
        } => {
            emit_function(em, name, *is_command, params, body, *line, None)?;
        }
        Stmt::MacroDef { .. } => {}
        Stmt::MacroCall { name, args, line } => {
            return Err(CompileError::new(
                "<input>",
                *line,
                format!("unexpanded macro '@{name}' with args {args:?}"),
            ));
        }
        Stmt::Bash { raw, .. } => {
            for line in raw.lines() {
                em.emit_line(&super::rewrite_refs(line));
            }
        }
    }
    Ok(())
}

pub fn emit_body(em: &mut Emitter, body: &[Stmt]) -> Result<()> {
    let mut i = 0;
    while i < body.len() {
        match &body[i] {
            Stmt::Function {
                is_command: true, ..
            } => {
                // nested commands handled inside emit_function
                i += 1;
            }
            stmt => {
                emit_stmt(em, stmt)?;
                i += 1;
            }
        }
    }
    Ok(())
}

fn expand_for_item(item: &str) -> String {
    let item = super::rewrite_refs(item);
    // $var alone -> "${var[@]}"
    let re = regex::Regex::new(r"^\$([A-Za-z_][A-Za-z0-9_]*)$").unwrap();
    if let Some(caps) = re.captures(&item) {
        return format!("\"${{{}[@]}}\"", &caps[1]);
    }
    item
}

fn emit_declare(
    em: &mut Emitter,
    name: &str,
    ty: &Type,
    value: Option<&Value>,
    line: usize,
) -> Result<()> {
    let _ = line;
    em.types.insert(name.to_string(), ty.clone());
    let mut flags = String::new();
    if ty.is_const {
        flags.push('r');
    }
    match ty.kind {
        TypeKind::Int => flags.push('i'),
        TypeKind::List => flags.push('a'),
        TypeKind::Dict => flags.push('A'),
        _ => {}
    }
    let val = value.map(emit_value);
    if flags.is_empty() {
        if let Some(v) = val {
            em.emit_line(&format!("{name}={v}"));
        } else {
            em.emit_line(&format!("declare {name}"));
        }
    } else if let Some(v) = val {
        em.emit_line(&format!("declare -{flags} {name}={v}"));
    } else {
        em.emit_line(&format!("declare -{flags} {name}"));
    }

    if let Some(v) = value {
        if !matches!(v, Value::List(_) | Value::Dict(_)) {
            if let Some(test) = type_test(&format!("${name}"), ty) {
                em.emit_line(&format!("if [[ {test} ]]; then"));
                em.emit_line(&format!(
                    "  echo \"{name}: expected {}, received ${name}\" >&2",
                    ty.description()
                ));
                em.emit_line("  exit 1");
                em.emit_line("fi");
            }
        }
    }
    Ok(())
}

fn emit_function(
    em: &mut Emitter,
    name: &str,
    is_command: bool,
    params: &[Param],
    body: &[Stmt],
    line: usize,
    parent: Option<&str>,
) -> Result<()> {
    let fn_name = if name.is_empty() {
        if em.depth == 0 {
            if em.root.is_some() {
                return Err(CompileError::new(
                    "<input>",
                    line,
                    "root command '.()' already defined",
                ));
            }
            let leaves = flatten_leaves(params);
            em.root = Some(if leaves.is_empty() {
                RootKind::Bare
            } else {
                RootKind::Params
            });
        }
        "_scrippo_root".to_string()
    } else if let Some(parent) = parent {
        format!("{parent}__{name}")
    } else {
        name.to_string()
    };

    let label = if name.is_empty() {
        "${0##*/}".to_string()
    } else {
        name.replace("__", " ")
    };

    if is_command && parent.is_none() && !name.is_empty() {
        if em.commands.iter().any(|c| c == name) {
            return Err(CompileError::new(
                "<input>",
                line,
                format!("command '{name}' already defined"),
            ));
        }
        em.commands.push(name.to_string());
    }

    // Emit nested command functions first
    let subcommands: Vec<&Stmt> = body
        .iter()
        .filter(|s| matches!(s, Stmt::Function { is_command: true, .. }))
        .collect();

    for sub in &subcommands {
        if let Stmt::Function {
            name: subname,
            params,
            body,
            line,
            ..
        } = sub
        {
            if name.is_empty() {
                return Err(CompileError::new(
                    "<input>",
                    *line,
                    "commands cannot be nested inside the root command '.()'",
                ));
            }
            if subname.is_empty() {
                return Err(CompileError::new(
                    "<input>",
                    *line,
                    "the root command '.()' must be defined at the top level",
                ));
            }
            emit_function(em, subname, true, params, body, *line, Some(&fn_name))?;
        }
    }

    em.emit_line(&format!("function {fn_name} {{"));
    em.indent();

    for sub in &subcommands {
        if let Stmt::Function { name: subname, .. } = sub {
            em.emit_line(&format!("if [[ $1 == {subname} ]]; then"));
            em.emit_line("  shift");
            em.emit_line(&format!("  {fn_name}__{subname} \"$@\""));
            em.emit_line("  return");
            em.emit_line("fi");
        }
    }

    emit_function_params(em, &label, params)?;

    for stmt in body {
        if matches!(stmt, Stmt::Function { is_command: true, .. }) {
            continue;
        }
        emit_stmt(em, stmt)?;
    }

    em.dedent();
    em.emit_line("}");
    if em.depth == 0 {
        em.emit_line("");
    }
    let _ = is_command;
    Ok(())
}
