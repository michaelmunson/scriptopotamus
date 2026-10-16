use crate::ast::*;
use crate::error::{CompileError, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub fn check_program(path: &Path, program: &Program) -> Result<()> {
    let mut commands = HashSet::new();
    let mut consts = HashMap::new();
    check_body(path, &program.body, &mut commands, &mut consts, false)
}

fn check_body(
    path: &Path,
    body: &[Stmt],
    commands: &mut HashSet<String>,
    consts: &mut HashMap<String, usize>,
    in_fn: bool,
) -> Result<()> {
    for stmt in body {
        match stmt {
            Stmt::Function {
                name,
                is_command,
                body,
                line,
                ..
            } => {
                if *is_command && !name.is_empty() && !in_fn {
                    if !commands.insert(name.clone()) {
                        return Err(CompileError::at(
                            path,
                            *line,
                            format!("command '{name}' already defined"),
                        ));
                    }
                }
                let mut nested = HashSet::new();
                check_body(path, body, &mut nested, consts, true)?;
            }
            Stmt::Declaration { name, ty, line } => {
                if ty.is_const {
                    consts.insert(name.clone(), *line);
                }
            }
            Stmt::Assignment { name, ty, line, .. } => {
                if let Some(ty) = ty {
                    if ty.is_const {
                        consts.insert(name.clone(), *line);
                    }
                } else if consts.contains_key(name) {
                    return Err(CompileError::at(
                        path,
                        *line,
                        format!("cannot reassign const '{name}'"),
                    ));
                }
            }
            Stmt::If {
                branches,
                else_body,
                ..
            } => {
                for b in branches {
                    check_body(path, &b.body, commands, consts, in_fn)?;
                }
                if let Some(body) = else_body {
                    check_body(path, body, commands, consts, in_fn)?;
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                check_body(path, body, commands, consts, in_fn)?;
            }
            Stmt::Case { arms, .. } => {
                for arm in arms {
                    check_body(path, &arm.body, commands, consts, in_fn)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
