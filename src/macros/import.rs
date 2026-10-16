use crate::ast::*;
use crate::error::{CompileError, Result};
use crate::parse;
use crate::paths;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const STD_IO: &str = include_str!("../../stdlib/io.scrippo");
const STD_TYPES: &str = include_str!("../../stdlib/types.scrippo");
const STD_PATH: &str = include_str!("../../stdlib/path.scrippo");
const STD_ALL: &str = include_str!("../../stdlib/std.scrippo");

pub struct ImportContext {
    pub imported: BTreeSet<PathBuf>,
    pub prelude_bash: Vec<String>,
    pub import_dir: PathBuf,
    pub source_path: PathBuf,
}

impl ImportContext {
    pub fn new(source: &Path) -> Self {
        let import_dir = source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let mut imported = BTreeSet::new();
        if let Ok(canon) = source.canonicalize() {
            imported.insert(canon);
        } else {
            imported.insert(source.to_path_buf());
        }
        Self {
            imported,
            prelude_bash: Vec::new(),
            import_dir,
            source_path: source.to_path_buf(),
        }
    }
}

pub fn process_imports(program: &mut Program, ctx: &mut ImportContext) -> Result<()> {
    let mut new_body = Vec::new();
    for stmt in std::mem::take(&mut program.body) {
        match stmt {
            Stmt::MacroCall {
                name,
                args,
                line,
            } if name == "import" => {
                if args.len() != 1 {
                    return Err(CompileError::at(
                        &ctx.source_path,
                        line,
                        "@import: expected a single <path>",
                    ));
                }
                let pattern = &args[0];
                if !is_literal_path(pattern) {
                    return Err(CompileError::at(
                        &ctx.source_path,
                        line,
                        format!("@import: <path> must be a literal, received {pattern}"),
                    ));
                }
                import_pattern(ctx, &mut new_body, pattern, line)?;
            }
            other => new_body.push(other),
        }
    }
    program.body = new_body;
    Ok(())
}

fn is_literal_path(s: &str) -> bool {
    let re = regex::Regex::new(r"^[A-Za-z0-9_./+*?\[\]-]+$").unwrap();
    re.is_match(s)
        || ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn import_pattern(
    ctx: &mut ImportContext,
    body: &mut Vec<Stmt>,
    pattern: &str,
    line: usize,
) -> Result<()> {
    let pattern = unquote(pattern);

    if let Some(stmts) = resolve_stdlib(&pattern)? {
        for stmt in stmts {
            merge_stmt(ctx, body, stmt, line)?;
        }
        return Ok(());
    }

    if paths::is_package_name(&pattern) {
        if let Some(pkg) = paths::resolve_package(&pattern, &ctx.source_path) {
            import_file(ctx, body, &pkg, line)?;
            return Ok(());
        }
        return Err(CompileError::at(
            &ctx.source_path,
            line,
            format!(
                "@import: package not found: {pattern} (looked in .scrippo/lib and ~/.scrippo/lib)"
            ),
        ));
    }

    let path = if Path::new(&pattern).is_absolute() {
        PathBuf::from(&pattern)
    } else {
        ctx.import_dir.join(&pattern)
    };

    if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
        import_glob(ctx, body, &path, line)?;
    } else {
        import_file(ctx, body, &path, line)?;
    }
    Ok(())
}

fn resolve_stdlib(pattern: &str) -> Result<Option<Vec<Stmt>>> {
    let virtual_path = Path::new("<stdlib>");
    match pattern {
        "scrippo/std" | "scrippo/std.scrippo" | "scrippo/*" | "scrippo/std/*" => {
            let mut all = Vec::new();
            let io = parse::parse_source(virtual_path, STD_IO)?;
            let types = parse::parse_source(virtual_path, STD_TYPES)?;
            let path = parse::parse_source(virtual_path, STD_PATH)?;
            all.extend(io.body);
            all.extend(types.body);
            all.extend(path.body);
            let _ = STD_ALL;
            Ok(Some(all))
        }
        "scrippo/std/io" | "scrippo/io" | "scrippo/std/io.scrippo" => {
            Ok(Some(parse::parse_source(virtual_path, STD_IO)?.body))
        }
        "scrippo/std/types" | "scrippo/types" | "scrippo/std/types.scrippo" => {
            Ok(Some(parse::parse_source(virtual_path, STD_TYPES)?.body))
        }
        "scrippo/std/path" | "scrippo/path" | "scrippo/std/path.scrippo" => {
            Ok(Some(parse::parse_source(virtual_path, STD_PATH)?.body))
        }
        _ => Ok(None),
    }
}

fn import_glob(
    ctx: &mut ImportContext,
    body: &mut Vec<Stmt>,
    pattern: &Path,
    line: usize,
) -> Result<()> {
    let pattern_str = pattern.to_string_lossy().to_string();
    let mut matches: Vec<PathBuf> = Vec::new();
    // Simple glob: use walk of parent + filter
    let parent = pattern.parent().unwrap_or(Path::new("."));
    let file_pat = pattern
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if let Ok(entries) = std::fs::read_dir(parent) {
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| glob_match(&file_pat, &p.file_name().unwrap().to_string_lossy()))
            .collect();
        files.sort();
        matches = files;
    }
    if matches.is_empty() {
        return Err(CompileError::at(
            &ctx.source_path,
            line,
            format!("@import: no matches: {pattern_str}"),
        ));
    }
    for file in matches {
        import_file(ctx, body, &file, line)?;
    }
    Ok(())
}

fn glob_match(pat: &str, name: &str) -> bool {
    let re = glob_to_regex(pat);
    regex::Regex::new(&re)
        .map(|r| r.is_match(name))
        .unwrap_or(false)
}

fn glob_to_regex(pat: &str) -> String {
    let mut out = String::from("^");
    for c in pat.chars() {
        match c {
            '*' => out.push_str(".*"),
            '?' => out.push('.'),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out.push('$');
    out
}

fn import_file(
    ctx: &mut ImportContext,
    body: &mut Vec<Stmt>,
    path: &Path,
    line: usize,
) -> Result<()> {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !ctx.imported.insert(canon.clone()) {
        return Ok(());
    }
    if !path.is_file() {
        return Err(CompileError::at(
            &ctx.source_path,
            line,
            format!("@import: file not found: {}", path.display()),
        ));
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "bash" || ext == "sh" {
        let text = std::fs::read_to_string(path).map_err(|e| {
            CompileError::at(
                &ctx.source_path,
                line,
                format!("@import: {}", e),
            )
        })?;
        ctx.prelude_bash.push(text);
        return Ok(());
    }

    let saved_dir = ctx.import_dir.clone();
    let saved_source = ctx.source_path.clone();
    ctx.import_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    ctx.source_path = path.to_path_buf();

    let mut prog = parse::parse_file(path)?;
    process_imports(&mut prog, ctx)?;

    ctx.import_dir = saved_dir;
    ctx.source_path = saved_source;

    for stmt in prog.body {
        merge_stmt(ctx, body, stmt, line)?;
    }
    Ok(())
}

fn merge_stmt(
    ctx: &mut ImportContext,
    body: &mut Vec<Stmt>,
    stmt: Stmt,
    line: usize,
) -> Result<()> {
    if let Stmt::Function {
        name,
        is_command: true,
        ..
    } = &stmt
    {
        if !name.is_empty()
            && body.iter().any(|s| {
                matches!(
                    s,
                    Stmt::Function {
                        name: n,
                        is_command: true,
                        ..
                    } if n == name
                )
            })
        {
            return Err(CompileError::at(
                &ctx.source_path,
                line,
                format!("command '{name}' already defined"),
            ));
        }
    }
    body.push(stmt);
    Ok(())
}

#[allow(dead_code)]
pub fn collect_commands(body: &[Stmt]) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for stmt in body {
        if let Stmt::Function {
            name,
            is_command: true,
            line,
            ..
        } = stmt
        {
            if !name.is_empty() {
                map.insert(name.clone(), *line);
            }
        }
    }
    map
}
