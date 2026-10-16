use super::{type_test, Emitter};
use crate::ast::*;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Leaf {
    pub kind: LeafKind,
    pub name: String,
    pub flags: Vec<String>,
    pub ty: Type,
    pub default: Option<String>,
    pub array: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafKind {
    Arg,
    Opt,
    Rest,
}

pub fn flatten_leaves(params: &[Param]) -> Vec<Leaf> {
    let mut out = Vec::new();
    flatten(params, false, &mut out);
    out
}

fn flatten(params: &[Param], parent_repeat: bool, out: &mut Vec<Leaf>) {
    for p in params {
        match p {
            Param::Arg {
                name,
                ty,
                optional,
                default,
                repeat,
            } => {
                let array = *repeat || parent_repeat;
                let _ = optional;
                out.push(Leaf {
                    kind: LeafKind::Arg,
                    name: name.clone(),
                    flags: Vec::new(),
                    ty: ty.clone(),
                    default: default.clone(),
                    array,
                    repeat: *repeat,
                });
            }
            Param::Opt {
                name,
                flags,
                ty,
                optional,
                default,
                repeat,
            } => {
                let _ = optional;
                let array = (*repeat || parent_repeat) && ty.kind != TypeKind::Bool;
                out.push(Leaf {
                    kind: LeafKind::Opt,
                    name: name.clone(),
                    flags: flags.clone(),
                    ty: ty.clone(),
                    default: default.clone(),
                    array,
                    repeat: *repeat,
                });
            }
            Param::Rest { name, ty } => {
                out.push(Leaf {
                    kind: LeafKind::Rest,
                    name: name.clone(),
                    flags: Vec::new(),
                    ty: ty.clone(),
                    default: None,
                    array: true,
                    repeat: true,
                });
            }
            Param::Group {
                params,
                optional: _,
            } => flatten(params, parent_repeat, out),
            Param::Exclusive { options } => {
                for opt in options {
                    flatten(std::slice::from_ref(opt), parent_repeat, out);
                }
            }
        }
    }
}

pub fn emit_function_params(em: &mut Emitter, fn_label: &str, params: &[Param]) -> Result<()> {
    let leaves = flatten_leaves(params);
    if leaves.is_empty() {
        return Ok(());
    }

    let mut catchall: Option<String> = None;
    for (i, leaf) in leaves.iter().enumerate() {
        if leaf.kind == LeafKind::Rest {
            if catchall.is_some() {
                // only one rest allowed - soft ignore second
            }
            if i != leaves.len() - 1 {
                // should be last
            }
            catchall = Some(leaf.name.clone());
        }
    }

    for leaf in &leaves {
        match leaf.kind {
            LeafKind::Rest => {
                em.emit_line(&format!("local -a {}=()", leaf.name));
            }
            LeafKind::Opt => {
                if leaf.array {
                    em.emit_line(&format!("local -a {}=()", leaf.name));
                } else if leaf.ty.kind == TypeKind::Bool {
                    let def = leaf.default.as_deref().unwrap_or("false");
                    em.emit_line(&format!("local {}={def}", leaf.name));
                } else if let Some(d) = &leaf.default {
                    em.emit_line(&format!("local {}={d}", leaf.name));
                } else {
                    em.emit_line(&format!("local {}=", leaf.name));
                }
            }
            LeafKind::Arg => {}
        }
    }

    if catchall.is_some() {
        em.emit_line("local -a _scrippo_seq=()");
        em.emit_line("local -a _scrippo_kind=()");
    }
    em.emit_line("local _scrippo_args=()");
    em.emit_line("while (( $# > 0 )); do");
    em.indent();
    em.emit_line("case $1 in");
    em.indent();

    for leaf in &leaves {
        if leaf.kind == LeafKind::Opt {
            emit_option(em, fn_label, leaf);
        }
    }

    if catchall.is_some() {
        em.emit_line("--)");
        em.emit_line("  shift");
        em.emit_line("  while (( $# > 0 )); do");
        em.emit_line("    _scrippo_seq+=(\"$1\")");
        em.emit_line("    _scrippo_kind+=(arg)");
        em.emit_line("    shift");
        em.emit_line("  done");
        em.emit_line("  ;;");
        em.emit_line("-?*)");
        em.emit_line("  _scrippo_seq+=(\"$1\")");
        em.emit_line("  _scrippo_kind+=(flag)");
        em.emit_line("  shift");
        em.emit_line("  ;;");
        em.emit_line("*)");
        em.emit_line("  _scrippo_seq+=(\"$1\")");
        em.emit_line("  _scrippo_kind+=(arg)");
        em.emit_line("  shift");
        em.emit_line("  ;;");
    } else {
        em.emit_line("--)");
        em.emit_line("  shift");
        em.emit_line("  _scrippo_args+=(\"$@\")");
        em.emit_line("  break");
        em.emit_line("  ;;");
        em.emit_line("-?*)");
        em.emit_line(&format!(
            "  echo \"{fn_label}: unknown option '$1'\" >&2"
        ));
        em.emit_line("  exit 1");
        em.emit_line("  ;;");
        em.emit_line("*)");
        em.emit_line("  _scrippo_args+=(\"$1\")");
        em.emit_line("  shift");
        em.emit_line("  ;;");
    }

    em.dedent();
    em.emit_line("esac");
    em.dedent();
    em.emit_line("done");

    emit_assign_args(em, fn_label, &leaves, catchall.as_deref())?;
    emit_checks(em, fn_label, params, &leaves)?;
    Ok(())
}

fn emit_option(em: &mut Emitter, fn_label: &str, leaf: &Leaf) {
    let pattern = leaf.flags.join("|");
    em.emit_line(&format!("{pattern})"));
    em.indent();
    if leaf.ty.kind == TypeKind::Bool && !leaf.array {
        em.emit_line(&format!("{}=true", leaf.name));
        em.emit_line("shift");
    } else if leaf.repeat {
        em.emit_line("shift");
        em.emit_line("while (( $# > 0 )) && [[ $1 != -?* ]]; do");
        em.emit_line(&format!("  {}+=(\"$1\")", leaf.name));
        em.emit_line("  shift");
        em.emit_line("done");
    } else {
        em.emit_line(&format!(
            "if (( $# < 2 )); then echo \"{fn_label}: {} requires a value\" >&2; exit 1; fi",
            leaf.flags.first().map(|s| s.as_str()).unwrap_or("")
        ));
        if leaf.array {
            em.emit_line(&format!("{}+=(\"$2\")", leaf.name));
        } else {
            em.emit_line(&format!("{}=$2", leaf.name));
        }
        em.emit_line("shift 2");
    }
    em.emit_line(";;");
    em.dedent();
}

fn emit_assign_args(
    em: &mut Emitter,
    fn_label: &str,
    leaves: &[Leaf],
    catchall: Option<&str>,
) -> Result<()> {
    if catchall.is_some() {
        em.emit_line("local _scrippo_i");
        em.emit_line("for _scrippo_i in \"${!_scrippo_seq[@]}\"; do");
        em.emit_line("  if [[ ${_scrippo_kind[_scrippo_i]} == arg ]]; then");
        em.emit_line("    _scrippo_args+=(\"${_scrippo_seq[_scrippo_i]}\")");
        em.emit_line("  fi");
        em.emit_line("done");
    }

    let mut position = 0usize;
    let mut rest = false;
    for leaf in leaves {
        if leaf.kind != LeafKind::Arg {
            continue;
        }
        if leaf.array {
            em.emit_line(&format!(
                "local -a {}=(\"${{_scrippo_args[@]:{position}}}\")",
                leaf.name
            ));
            rest = true;
        } else if let Some(d) = &leaf.default {
            em.emit_line(&format!(
                "local {}=${{_scrippo_args[{position}]:-{d}}}",
                leaf.name
            ));
            position += 1;
        } else {
            em.emit_line(&format!(
                "local {}=${{_scrippo_args[{position}]}}",
                leaf.name
            ));
            position += 1;
        }
    }

    if let Some(name) = catchall {
        if rest {
            em.emit_line("for _scrippo_i in \"${!_scrippo_seq[@]}\"; do");
            em.emit_line("  if [[ ${_scrippo_kind[_scrippo_i]} == flag ]]; then");
            em.emit_line(&format!("    {name}+=(\"${{_scrippo_seq[_scrippo_i]}}\")"));
            em.emit_line("  fi");
            em.emit_line("done");
        } else if position == 0 {
            em.emit_line(&format!("{name}=(\"${{_scrippo_seq[@]}}\")"));
        } else {
            em.emit_line("local _scrippo_seen=0");
            em.emit_line("for _scrippo_i in \"${!_scrippo_seq[@]}\"; do");
            em.emit_line(&format!(
                "  if [[ ${{_scrippo_kind[_scrippo_i]}} == arg ]] && (( _scrippo_seen < {position} )); then"
            ));
            em.emit_line("    _scrippo_seen=$(( _scrippo_seen + 1 ))");
            em.emit_line("    continue");
            em.emit_line("  fi");
            em.emit_line(&format!("  {name}+=(\"${{_scrippo_seq[_scrippo_i]}}\")"));
            em.emit_line("done");
        }
    } else if !rest {
        em.emit_line(&format!(
            "if (( ${{#_scrippo_args[@]}} > {position} )); then echo \"{fn_label}: too many arguments\" >&2; exit 1; fi"
        ));
    }
    Ok(())
}

fn used_expr(param: &Param) -> String {
    match param {
        Param::Group { params, .. } => {
            let parts: Vec<_> = params.iter().map(used_expr).collect();
            format!("( {} )", parts.join(" || "))
        }
        Param::Exclusive { options } => {
            let parts: Vec<_> = options.iter().map(used_expr).collect();
            format!("( {} )", parts.join(" || "))
        }
        Param::Rest { name, .. } => format!("${{#{name}[@]}} -gt 0"),
        Param::Arg {
            name,
            ty,
            repeat,
            ..
        } => {
            if *repeat || ty.kind == TypeKind::List {
                format!("${{#{name}[@]}} -gt 0")
            } else {
                format!("-n ${name}")
            }
        }
        Param::Opt {
            name,
            ty,
            repeat,
            ..
        } => {
            if *repeat {
                format!("${{#{name}[@]}} -gt 0")
            } else if ty.kind == TypeKind::Bool {
                format!("${name} == true")
            } else if ty.kind == TypeKind::List {
                format!("${{#{name}[@]}} -gt 0")
            } else {
                format!("-n ${name}")
            }
        }
    }
}

fn name_of(param: &Param) -> String {
    match param {
        Param::Group { params, .. } => {
            let inner: Vec<_> = params.iter().map(name_of).collect();
            format!("[{}]", inner.join(" "))
        }
        Param::Exclusive { options } => options
            .iter()
            .map(name_of)
            .collect::<Vec<_>>()
            .join(" | "),
        Param::Rest { name, .. } => format!("*{name}"),
        Param::Arg { name, .. } => format!("<{name}>"),
        Param::Opt { flags, .. } => flags.first().cloned().unwrap_or_default(),
    }
}

struct Check {
    test: String,
    message: String,
}

fn collect_checks(params: &[Param], context: Option<&str>, out: &mut Vec<Check>) {
    for p in params {
        match p {
            Param::Group { params, optional } => {
                if *optional {
                    let used = used_expr(p);
                    let ctx = match context {
                        Some(c) => format!("{c} && {used}"),
                        None => used,
                    };
                    collect_checks(params, Some(&ctx), out);
                } else {
                    collect_checks(params, context, out);
                }
            }
            Param::Exclusive { options } => {
                for i in 0..options.len() {
                    for j in (i + 1)..options.len() {
                        let test = when(
                            context,
                            &format!("{} && {}", used_expr(&options[i]), used_expr(&options[j])),
                        );
                        out.push(Check {
                            test,
                            message: format!(
                                "{} and {} are mutually exclusive",
                                name_of(&options[i]),
                                name_of(&options[j])
                            ),
                        });
                    }
                }
                if !options.iter().any(|o| o.is_optional()) {
                    out.push(Check {
                        test: when(context, &format!("! {}", used_expr(p))),
                        message: format!("one of {} is required", name_of(p)),
                    });
                }
                for opt in options {
                    if let Param::Group { params, .. } = opt {
                        let used = used_expr(opt);
                        let ctx = match context {
                            Some(c) => format!("{c} && {used}"),
                            None => used,
                        };
                        collect_checks(params, Some(&ctx), out);
                    }
                }
            }
            Param::Arg { optional, .. } | Param::Opt { optional, .. } => {
                if !*optional {
                    let used = used_expr(p);
                    let test = if used.starts_with("-n ") {
                        when(context, &format!("-z {}", &used[3..]))
                    } else if used.contains("== true") {
                        when(context, &format!("! ({used})"))
                    } else {
                        when(context, &format!("! ({used})"))
                    };
                    out.push(Check {
                        test,
                        message: format!("{} is required", name_of(p)),
                    });
                }
            }
            Param::Rest { .. } => {}
        }
    }
}

fn when(context: Option<&str>, test: &str) -> String {
    match context {
        Some(c) => format!("{c} && {test}"),
        None => test.to_string(),
    }
}

fn emit_checks(em: &mut Emitter, fn_label: &str, params: &[Param], leaves: &[Leaf]) -> Result<()> {
    let mut checks = Vec::new();
    collect_checks(params, None, &mut checks);
    for check in checks {
        em.emit_line(&format!(
            "if [[ {} ]]; then echo \"{fn_label}: {}\" >&2; exit 1; fi",
            check.test, check.message
        ));
    }

    for leaf in leaves {
        if leaf.kind == LeafKind::Opt && leaf.ty.kind == TypeKind::Bool {
            continue;
        }
        let flag = match leaf.kind {
            LeafKind::Rest => format!("*{}", leaf.name),
            LeafKind::Arg => format!("<{}>", leaf.name),
            LeafKind::Opt => leaf.flags.first().cloned().unwrap_or_default(),
        };
        if leaf.array {
            if let Some(test) = type_test("$_scrippo_item", &leaf.ty) {
                em.emit_line(&format!("for _scrippo_item in \"${{{}[@]}}\"; do", leaf.name));
                em.emit_line(&format!(
                    "  if [[ {test} ]]; then echo \"{fn_label}: {flag} expected {}, received '$_scrippo_item'\" >&2; exit 1; fi",
                    leaf.ty.description()
                ));
                em.emit_line("done");
            }
        } else if let Some(test) = type_test(&format!("${}", leaf.name), &leaf.ty) {
            em.types.insert(leaf.name.clone(), leaf.ty.clone());
            em.emit_line(&format!(
                "if [[ -n ${} && {test} ]]; then echo \"{fn_label}: {flag} expected {}, received '${}'\" >&2; exit 1; fi",
                leaf.name,
                leaf.ty.description(),
                leaf.name
            ));
        }
    }
    Ok(())
}
