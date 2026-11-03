use super::import::{ImportContext, process_imports};
use crate::ast::*;
use crate::error::{CompileError, Result};
use std::path::Path;

pub struct ExpandResult {
    pub program: Program,
    pub prelude_bash: Vec<String>,
}

pub fn expand_program(path: &Path, program: Program) -> Result<ExpandResult> {
    let ctx = ImportContext::new(path);
    expand_with_context(path, program, ctx)
}

pub(crate) fn expand_program_sandboxed(
    path: &Path,
    program: Program,
    root: &Path,
) -> Result<ExpandResult> {
    let mut ctx = ImportContext::new(path);
    ctx.sandbox_root = Some(root.to_path_buf());
    ctx.check_path(path, 0)?;
    expand_with_context(path, program, ctx)
}

fn expand_with_context(
    path: &Path,
    mut program: Program,
    mut ctx: ImportContext,
) -> Result<ExpandResult> {
    process_imports(&mut program, &mut ctx)?;

    program.body = expand_body(&program.body)?;
    inherit_params(path, &mut program.body, None)?;

    Ok(ExpandResult {
        program,
        prelude_bash: ctx.prelude_bash,
    })
}

fn expand_body(body: &[Stmt]) -> Result<Block> {
    let mut out = Vec::new();
    for stmt in body {
        match stmt {
            Stmt::MacroCall { name, .. } if name == "usage" => out.push(stmt.clone()),
            Stmt::Hook {
                name,
                args,
                body,
                line,
            } => {
                out.push(Stmt::Hook {
                    name: name.clone(),
                    args: args.clone(),
                    body: expand_body(body)?,
                    line: *line,
                });
            }
            Stmt::MacroCall { name, line, .. } => {
                let message = if name == "import" {
                    "@import remaining after import pass".to_string()
                } else if name == "hook" {
                    "@hook must be written on its own line followed by an indented body".to_string()
                } else {
                    format!("unknown macro '@{name}'")
                };
                return Err(CompileError::new("<input>", *line, message));
            }
            Stmt::If {
                branches,
                else_body,
                line,
            } => {
                let mut new_branches = Vec::new();
                for b in branches {
                    new_branches.push(IfBranch {
                        condition: b.condition.clone(),
                        body: expand_body(&b.body)?,
                    });
                }
                let else_body = match else_body {
                    Some(b) => Some(expand_body(b)?),
                    None => None,
                };
                out.push(Stmt::If {
                    branches: new_branches,
                    else_body,
                    line: *line,
                });
            }
            Stmt::For {
                var,
                items,
                body,
                line,
            } => {
                out.push(Stmt::For {
                    var: var.clone(),
                    items: items.clone(),
                    body: expand_body(body)?,
                    line: *line,
                });
            }
            Stmt::While {
                condition,
                body,
                line,
            } => {
                out.push(Stmt::While {
                    condition: condition.clone(),
                    body: expand_body(body)?,
                    line: *line,
                });
            }
            Stmt::Case {
                subject,
                arms,
                line,
            } => {
                let mut new_arms = Vec::new();
                for arm in arms {
                    new_arms.push(CaseArm {
                        pattern: arm.pattern.clone(),
                        body: expand_body(&arm.body)?,
                        terminator: arm.terminator.clone(),
                    });
                }
                out.push(Stmt::Case {
                    subject: subject.clone(),
                    arms: new_arms,
                    line: *line,
                });
            }
            Stmt::Function {
                name,
                is_command,
                docs,
                params,
                body,
                line,
            } => {
                out.push(Stmt::Function {
                    name: name.clone(),
                    is_command: *is_command,
                    docs: docs.clone(),
                    params: params.clone(),
                    body: expand_body(body)?,
                    line: *line,
                });
            }
            other => out.push(other.clone()),
        }
    }
    Ok(out)
}

fn inherit_params(
    path: &Path,
    body: &mut [Stmt],
    parent: Option<(&[Param], &[(String, String)])>,
) -> Result<()> {
    for stmt in body {
        if let Stmt::Function {
            params,
            docs,
            body,
            line,
            ..
        } = stmt
        {
            let mut resolved = Vec::new();
            for param in std::mem::take(params) {
                if matches!(&param, Param::Arg { name, .. } if name == "@inherit") {
                    let (inherited, comments) = parent.ok_or_else(|| {
                        CompileError::at(
                            path,
                            *line,
                            "@inherit requires a parent command or function",
                        )
                    })?;
                    resolved.extend_from_slice(inherited);
                    docs.extend(comments.iter().filter(|(key, _)| !key.is_empty()).cloned());
                } else {
                    resolved.push(param);
                }
            }
            *params = resolved;
            inherit_params(path, body, Some((params, docs)))?;
        }
    }
    Ok(())
}
