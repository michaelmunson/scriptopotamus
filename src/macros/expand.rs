use super::import::{process_imports, ImportContext};
use crate::ast::*;
use crate::error::{CompileError, Result};
use std::collections::HashMap;
use std::path::Path;

pub struct ExpandResult {
    pub program: Program,
    pub prelude_bash: Vec<String>,
}

pub fn expand_program(path: &Path, mut program: Program) -> Result<ExpandResult> {
    let mut ctx = ImportContext::new(path);
    process_imports(&mut program, &mut ctx)?;

    let mut macros: HashMap<String, MacroDef> = HashMap::new();
    collect_macros(&program.body, &mut macros);
    program.body = expand_body(&program.body, &macros)?;

    Ok(ExpandResult {
        program,
        prelude_bash: ctx.prelude_bash,
    })
}

struct MacroDef {
    body: Block,
}

fn collect_macros(body: &[Stmt], macros: &mut HashMap<String, MacroDef>) {
    for stmt in body {
        match stmt {
            Stmt::MacroDef { name, body, .. } => {
                macros.insert(
                    name.clone(),
                    MacroDef {
                        body: body.clone(),
                    },
                );
            }
            Stmt::Function { body, .. }
            | Stmt::If {
                branches: _,
                else_body: Some(body),
                ..
            } => {
                collect_macros(body, macros);
            }
            Stmt::If { branches, else_body, .. } => {
                for b in branches {
                    collect_macros(&b.body, macros);
                }
                if let Some(body) = else_body {
                    collect_macros(body, macros);
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                collect_macros(body, macros);
            }
            Stmt::Case { arms, .. } => {
                for arm in arms {
                    collect_macros(&arm.body, macros);
                }
            }
            _ => {}
        }
    }
}

fn expand_body(body: &[Stmt], macros: &HashMap<String, MacroDef>) -> Result<Block> {
    let mut out = Vec::new();
    for stmt in body {
        match stmt {
            Stmt::MacroDef { .. } => {}
            Stmt::MacroCall { name, args, line } => {
                if name == "import" {
                    return Err(CompileError::new(
                        "<input>",
                        *line,
                        "@import remaining after import pass",
                    ));
                }
                let def = macros.get(name).ok_or_else(|| {
                    CompileError::new("<input>", *line, format!("unknown macro '@{name}'"))
                })?;
                let _ = args;
                let expanded = expand_body(&def.body, macros)?;
                out.extend(expanded);
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
                        body: expand_body(&b.body, macros)?,
                    });
                }
                let else_body = match else_body {
                    Some(b) => Some(expand_body(b, macros)?),
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
                    body: expand_body(body, macros)?,
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
                    body: expand_body(body, macros)?,
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
                        body: expand_body(&arm.body, macros)?,
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
                params,
                body,
                line,
            } => {
                out.push(Stmt::Function {
                    name: name.clone(),
                    is_command: *is_command,
                    params: params.clone(),
                    body: expand_body(body, macros)?,
                    line: *line,
                });
            }
            Stmt::Bash { raw, line } => {
                out.push(Stmt::Bash {
                    raw: expand_bash_macros(raw, macros)?,
                    line: *line,
                });
            }
            other => out.push(other.clone()),
        }
    }
    Ok(out)
}

fn expand_bash_macros(raw: &str, macros: &HashMap<String, MacroDef>) -> Result<String> {
    let mut result = String::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('@') {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if let Some(def) = macros.get(&name) {
                let expanded = expand_body(&def.body, macros)?;
                // Emit expanded stmts as approximate bash lines via echo/raw
                for stmt in expanded {
                    match stmt {
                        Stmt::Echo { value, .. } => {
                            let v = match value {
                                crate::ast::Value::Scalar(s) => s,
                                other => format!("{other:?}"),
                            };
                            result.push_str(&format!("echo {v}\n"));
                        }
                        Stmt::Bash { raw, .. } => {
                            result.push_str(&raw);
                            result.push('\n');
                        }
                        _ => {
                            result.push_str(&format!("# expanded macro stmt\n"));
                        }
                    }
                }
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    if result.ends_with('\n') {
        result.pop();
    }
    Ok(result)
}
