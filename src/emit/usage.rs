use super::{
    Emitter,
    params::{LeafKind, flatten_leaves},
};
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
    let mut view = View {
        docs,
        colored,
        out: format!(
            "  {}{}\n",
            paint(name, 32, colored),
            comments(docs, "", colored)
        ),
        bullet: false,
    };
    view.params(params, 4);
    view.out
}

/// Renders a parameter tree: nested groups indent beneath their leading option,
/// and every alternative of an exclusive set is bulleted under `one of:`.
struct View<'a> {
    docs: &'a [(String, String)],
    colored: bool,
    out: String,
    bullet: bool,
}

impl View<'_> {
    fn line(&mut self, indent: usize, text: &str) {
        if std::mem::take(&mut self.bullet) {
            self.out
                .push_str(&format!("{}- {text}\n", " ".repeat(indent.saturating_sub(2))));
        } else {
            self.out.push_str(&format!("{}{text}\n", " ".repeat(indent)));
        }
    }

    fn params(&mut self, params: &[Param], indent: usize) {
        for param in params {
            self.param(param, indent);
        }
    }

    fn param(&mut self, param: &Param, indent: usize) {
        match param {
            Param::Arg { .. } | Param::Opt { .. } | Param::Rest { .. } => self.leaf(param, indent),
            Param::Exclusive { options } => {
                let header = paint("one of:", 2, self.colored);
                self.line(indent, &header);
                for option in options {
                    self.bullet = true;
                    self.param(option, indent + 4);
                }
            }
            Param::Group { params, .. } => match params.first() {
                Some(lead) if is_leaf(lead) => {
                    self.leaf(lead, indent);
                    self.params(&params[1..], indent + 2);
                }
                _ => self.params(params, indent),
            },
        }
    }

    fn leaf(&mut self, param: &Param, indent: usize) {
        let Some(leaf) = flatten_leaves(std::slice::from_ref(param)).into_iter().next() else {
            return;
        };
        let option = !leaf.flags.is_empty();
        let label = if option {
            leaf.flags.join(" / ")
        } else {
            leaf.name.clone()
        };
        let mut label = if param.is_optional() {
            format!("[{label}]")
        } else {
            format!("{{{label}}}")
        };
        if leaf.repeat {
            label.push_str("...");
        }
        let mut text = paint(&label, if option { 34 } else { 36 }, self.colored);
        if leaf.kind != LeafKind::Rest
            && leaf.ty.kind
                != if option {
                    TypeKind::Bool
                } else {
                    TypeKind::Str
                }
        {
            text.push_str(&paint(&format!("<{}>", leaf.ty.description()), 33, self.colored));
        }
        let key = if option { &leaf.flags[0] } else { &leaf.name };
        text.push_str(&comments(self.docs, key, self.colored));
        self.line(indent, &text);
    }
}

fn is_leaf(param: &Param) -> bool {
    matches!(param, Param::Arg { .. } | Param::Opt { .. } | Param::Rest { .. })
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
    em.emit_verbatim(&format!(
        "{prefix}_scrippo_usage_text={}",
        quote(&text(name, params, docs, body, false))
    ));
    em.emit_verbatim(&format!(
        "{prefix}_scrippo_usage_color={}",
        quote(&text(name, params, docs, body, true))
    ));
}
pub fn has_help(params: &[Param]) -> bool {
    flatten_leaves(params)
        .iter()
        .any(|l| l.flags.iter().any(|f| f == "--help"))
}
