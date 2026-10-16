pub mod ast;
pub mod check;
pub mod config;
pub mod emit;
pub mod error;
pub mod macros;
pub mod parse;
pub mod paths;
pub mod shell;

use crate::error::Result;
use std::path::Path;

pub fn compile_file(path: &Path) -> Result<String> {
    let program = parse::parse_file(path)?;
    compile_program(path, program)
}

pub fn compile_source(path: &Path, source: &str) -> Result<String> {
    let program = parse::parse_source(path, source)?;
    compile_program(path, program)
}

pub fn compile_program(path: &Path, program: ast::Program) -> Result<String> {
    let expanded = macros::expand_program(path, program)?;
    check::check_program(path, &expanded.program)?;
    let mut bash = emit::emit_program(&expanded.program)?;

    let needs_helpers = program_needs_helpers(&expanded.program);
    if needs_helpers || !expanded.prelude_bash.is_empty() {
        let helpers = include_str!("../stdlib/helpers.bash");
        let mut parts = Vec::new();
        parts.push("#!/usr/bin/env bash".to_string());
        if needs_helpers {
            parts.push(helpers.trim_end().to_string());
            parts.push(String::new());
        }
        for prelude in &expanded.prelude_bash {
            parts.push(prelude.trim_end().to_string());
            parts.push(String::new());
        }
        // Strip existing shebang from emitted bash
        let body = bash
            .strip_prefix("#!/usr/bin/env bash\n")
            .unwrap_or(&bash);
        parts.push(body.trim_end().to_string());
        bash = parts.join("\n") + "\n";
    }

    Ok(bash)
}

fn program_needs_helpers(program: &ast::Program) -> bool {
    fn walk(stmts: &[ast::Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                ast::Stmt::Function { name, body, .. } => {
                    if matches!(name.as_str(), "prt" | "input" | "throw") {
                        return true;
                    }
                    if walk(body) {
                        return true;
                    }
                }
                ast::Stmt::If {
                    branches,
                    else_body,
                    ..
                } => {
                    for b in branches {
                        if walk(&b.body) {
                            return true;
                        }
                    }
                    if let Some(body) = else_body {
                        if walk(body) {
                            return true;
                        }
                    }
                }
                ast::Stmt::For { body, .. } | ast::Stmt::While { body, .. } => {
                    if walk(body) {
                        return true;
                    }
                }
                ast::Stmt::Case { arms, .. } => {
                    for arm in arms {
                        if walk(&arm.body) {
                            return true;
                        }
                    }
                }
                ast::Stmt::Bash { raw, .. } => {
                    if raw.contains("_scrippo_") {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
    walk(&program.body)
}
