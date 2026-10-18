use super::{Emitter, params::flatten_leaves};
use crate::ast::*;

pub fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn paint(s: &str, color: u8, colored: bool) -> String {
    if colored {
        format!("\x1b[{color}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}
fn comments(docs: &[(String, String)], key: &str, colored: bool) -> String {
    let text = docs
        .iter()
        .filter(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    if text.is_empty() {
        String::new()
    } else {
        format!(" # {}", paint(&text, 2, colored))
    }
}
fn entry(name: &str, params: &[Param], docs: &[(String, String)], colored: bool) -> String {
    let mut text = format!(
        "  {}{}\n",
        paint(name, 32, colored),
        comments(docs, "", colored)
    );
    append_exclusives(&mut text, params, colored);
    for leaf in flatten_leaves(params) {
        let option = !leaf.flags.is_empty();
        let label = if option {
            leaf.flags.join(" / ")
        } else {
            leaf.name.clone()
        };
        let optional = optional_leaf(params, &leaf.name);
        let mut label = if optional {
            format!("[{label}]")
        } else {
            format!("{{{label}}}")
        };
        if leaf.repeat {
            label.push_str("...");
        }
        text.push_str(&format!(
            "    {}",
            paint(&label, if option { 34 } else { 36 }, colored)
        ));
        if leaf.ty.kind
            != if option {
                TypeKind::Bool
            } else {
                TypeKind::Str
            }
        {
            text.push_str(&paint(&format!("<{}>", leaf.ty.description()), 33, colored));
        }
        let key = if option { &leaf.flags[0] } else { &leaf.name };
        text.push_str(&comments(docs, key, colored));
        text.push('\n');
    }
    text
}
fn append_exclusives(text: &mut String, params: &[Param], colored: bool) {
    for param in params {
        match param {
            Param::Exclusive { options } => {
                let alternatives = options
                    .iter()
                    .map(|option| {
                        flatten_leaves(std::slice::from_ref(option))
                            .iter()
                            .map(|leaf| {
                                if leaf.flags.is_empty() {
                                    paint(&leaf.name, 36, colored)
                                } else {
                                    paint(&leaf.flags.join(" / "), 34, colored)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                text.push_str(&format!("    {alternatives} (mutually exclusive)\n"));
            }
            Param::Group { params, .. } => append_exclusives(text, params, colored),
            _ => {}
        }
    }
}
fn optional_leaf(params: &[Param], name: &str) -> bool {
    params.iter().any(|p| match p {
        Param::Arg {
            name: n, optional, ..
        }
        | Param::Opt {
            name: n, optional, ..
        } => n == name && *optional,
        Param::Rest { name: n, .. } => n == name,
        Param::Group { params, optional } => {
            (*optional && flatten_leaves(params).iter().any(|l| l.name == name))
                || optional_leaf(params, name)
        }
        Param::Exclusive { options } => optional_leaf(options, name),
    })
}
pub fn text(
    name: &str,
    params: &[Param],
    docs: &[(String, String)],
    body: &[Stmt],
    colored: bool,
) -> String {
    let mut result = String::from("Usage:\n");
    if !name.is_empty() {
        result.push_str(&entry(name, params, docs, colored));
    }
    append_commands(&mut result, name, body, colored);
    result.trim_end().to_string()
}
fn append_commands(result: &mut String, name: &str, body: &[Stmt], colored: bool) {
    for stmt in body {
        if let Stmt::Function {
            name: sub,
            is_command: true,
            params,
            docs,
            body,
            ..
        } = stmt
        {
            let path = if name.is_empty() {
                sub.clone()
            } else {
                format!("{name} {sub}")
            };
            result.push_str(&entry(&path, params, docs, colored));
            append_commands(result, &path, body, colored);
        }
    }
}
pub fn bind(
    em: &mut Emitter,
    name: &str,
    params: &[Param],
    docs: &[(String, String)],
    body: &[Stmt],
    local: bool,
) {
    let prefix = if local { "local " } else { "" };
    em.emit_line(&format!(
        "{prefix}_scrippo_usage_text={}",
        quote(&text(name, params, docs, body, false))
    ));
    em.emit_line(&format!(
        "{prefix}_scrippo_usage_color={}",
        quote(&text(name, params, docs, body, true))
    ));
}
pub fn has_help(params: &[Param]) -> bool {
    flatten_leaves(params)
        .iter()
        .any(|l| l.flags.iter().any(|f| f == "--help"))
}
