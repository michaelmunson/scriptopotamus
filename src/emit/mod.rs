mod params;
mod stmt;
mod usage;

use crate::ast::*;
use crate::error::Result;
use std::collections::{BTreeMap, BTreeSet};

pub struct Emitter {
    pub lines: Vec<String>,
    pub prelude: Vec<String>,
    pub helpers: BTreeSet<String>,
    pub helper_bodies: BTreeMap<String, String>,
    pub depth: usize,
    pub commands: Vec<String>,
    pub root: Option<RootKind>,
    pub types: BTreeMap<String, Type>,
    pub needs_input_helpers: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    Bare,
    Params,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            prelude: Vec::new(),
            helpers: BTreeSet::new(),
            helper_bodies: BTreeMap::new(),
            depth: 0,
            commands: Vec::new(),
            root: None,
            types: BTreeMap::new(),
            needs_input_helpers: false,
        }
    }

    pub fn emit_line(&mut self, line: &str) {
        let pad = "  ".repeat(self.depth);
        for l in line.lines() {
            self.lines.push(format!("{pad}{l}"));
        }
    }

    pub fn indent(&mut self) {
        self.depth += 1;
    }

    pub fn dedent(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn add_helper(&mut self, name: &str, body: &str) {
        if self.helpers.insert(name.to_string()) {
            self.helper_bodies
                .insert(name.to_string(), body.to_string());
        }
    }

    pub fn finish(self) -> String {
        let mut result = vec!["#!/usr/bin/env bash".to_string()];
        for body in self.helper_bodies.values() {
            result.push(body.trim_end().to_string());
            result.push(String::new());
        }
        result.extend(self.prelude);
        result.extend(self.lines);
        result.join("\n") + "\n"
    }
}

pub fn emit_program(program: &Program) -> Result<String> {
    let mut em = Emitter::new();
    em.add_helper(
        "usage",
        r#"_scrippo_usage() {
  if [[ -t 1 && -z ${NO_COLOR+x} ]]; then
    printf '%s\n' "$_scrippo_usage_color"
  else
    printf '%s\n' "$_scrippo_usage_text"
  fi
}"#,
    );
    let root = program.body.iter().find_map(|s| match s {
        Stmt::Function {
            name,
            params,
            docs,
            body,
            ..
        } if name.is_empty() => Some((params, docs, body)),
        _ => None,
    });
    if let Some((params, docs, body)) = root {
        usage::bind(&mut em, "script", params, docs, body, false);
    } else {
        usage::bind(&mut em, "", &[], &[], &program.body, false);
    }
    if !root.is_some_and(|(params, _, _)| usage::has_help(params)) {
        em.emit_line("if [[ ${1-} == --help ]]; then _scrippo_usage; exit 0; fi");
    }
    for stmt in &program.body {
        stmt::emit_stmt(&mut em, stmt)?;
    }
    emit_dispatch(&mut em);
    Ok(em.finish())
}

fn emit_dispatch(em: &mut Emitter) {
    if em.root.is_none() && em.commands.is_empty() {
        return;
    }
    em.emit_line("case $1 in");
    em.indent();
    match em.root {
        None => {
            em.emit_line("\"\")");
            em.emit_line("  _scrippo_usage >&2");
            em.emit_line("  exit 1");
            em.emit_line("  ;;");
        }
        Some(RootKind::Bare) => {
            em.emit_line("\"\")");
            em.emit_line("  _scrippo_root");
            em.emit_line("  ;;");
        }
        Some(RootKind::Params) => {}
    }
    for cmd in &em.commands.clone() {
        em.emit_line(&format!("{cmd})"));
        em.emit_line("  shift");
        em.emit_line(&format!("  {cmd} \"$@\""));
        em.emit_line("  ;;");
    }
    em.emit_line("*)");
    if em.root == Some(RootKind::Params) {
        em.emit_line("  _scrippo_root \"$@\"");
    } else {
        em.emit_line("  echo \"${0##*/}: unknown command '$1'\" >&2");
        if !em.commands.is_empty() {
            em.emit_line(&format!(
                "  echo \"commands: {}\" >&2",
                em.commands.join(" ")
            ));
        }
        em.emit_line("  _scrippo_usage >&2");
        em.emit_line("  exit 1");
    }
    em.emit_line("  ;;");
    em.dedent();
    em.emit_line("esac");
}

pub fn emit_value(value: &Value) -> String {
    match value {
        Value::Scalar(s) => rewrite_refs(s),
        Value::List(items) => {
            let inner = items
                .iter()
                .map(|i| rewrite_refs(i))
                .collect::<Vec<_>>()
                .join(" ");
            format!("({inner})")
        }
        Value::Dict(entries) => {
            let inner = entries
                .iter()
                .map(|(k, v)| format!("[{}]={}", rewrite_refs(k), rewrite_refs(v)))
                .collect::<Vec<_>>()
                .join(" ");
            format!("({inner})")
        }
        Value::Math(raw) => {
            let expr = rewrite_refs(raw);
            format!("$(bc <<< \"{expr}\")")
        }
    }
}

pub fn rewrite_refs(s: &str) -> String {
    // $name[key] -> ${name[key]}, $--opt -> $opt-style already handled as var names
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                // already braced
                out.push('$');
                i += 1;
                continue;
            }
            // $--flag or $-f or $name or $name[idx]
            let start = i + 1;
            let mut j = start;
            if j < chars.len() && chars[j] == '-' {
                while j < chars.len()
                    && (chars[j] == '-' || chars[j].is_alphanumeric() || chars[j] == '_')
                {
                    j += 1;
                }
                let flag = chars[start..j].iter().collect::<String>();
                let name = flag.trim_start_matches('-').replace('-', "_");
                out.push('$');
                out.push_str(&name);
                i = j;
                continue;
            }
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            if j < chars.len() && chars[j] == '[' {
                let name: String = chars[start..j].iter().collect();
                let mut k = j + 1;
                while k < chars.len() && chars[k] != ']' {
                    k += 1;
                }
                let key: String = chars[j + 1..k].iter().collect();
                out.push_str(&format!("${{{name}[{key}]}}"));
                i = if k < chars.len() { k + 1 } else { k };
                continue;
            }
            out.push('$');
            out.extend(chars[start..j].iter());
            i = j;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn emit_condition(cond: &Condition) -> String {
    let raw = rewrite_refs(&cond.raw);
    let inner = match cond.kind {
        ConditionKind::Bool => format!("[[ {raw} == true ]]"),
        ConditionKind::Command => raw.clone(),
        ConditionKind::Test => {
            if raw.starts_with("[[") || raw.starts_with("[") {
                raw.clone()
            } else {
                format!("[[ {raw} ]]")
            }
        }
    };
    if cond.negated {
        format!("! {inner}")
    } else {
        inner
    }
}

pub fn type_test(value_expr: &str, ty: &Type) -> Option<String> {
    let base = match ty.kind {
        TypeKind::Int => Some(format!("! {value_expr} =~ ^-?[0-9]+$")),
        TypeKind::Float => Some(format!("! {value_expr} =~ ^-?[0-9]*\\.?[0-9]+$")),
        TypeKind::Bool => Some(format!("! {value_expr} =~ ^(true|false)$")),
        TypeKind::Str | TypeKind::List | TypeKind::Dict | TypeKind::Any => None,
    };
    if let Some(lits) = &ty.literals {
        let alt = lits
            .iter()
            .map(|l| regex::escape(l.trim_matches('"')))
            .collect::<Vec<_>>()
            .join("|");
        let lit_test = format!("! {value_expr} =~ ^({alt})$");
        return Some(match base {
            Some(b) => format!("({b}) || ({lit_test})"),
            None => lit_test,
        });
    }
    base
}
