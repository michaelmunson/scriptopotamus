use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Override with `SCRIPPO_HOME`. Default: `$HOME/.scrippo`.
pub fn scrippo_home() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("SCRIPPO_HOME") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".scrippo"))
}

pub fn global_lib_dir() -> Result<PathBuf> {
    Ok(scrippo_home()?.join("lib"))
}

pub fn global_scripts_dir() -> Result<PathBuf> {
    Ok(scrippo_home()?.join("scripts"))
}

/// Walk upward from `start` looking for a `.scrippo` directory.
pub fn find_project_scrippo_dir(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = dir.join(".scrippo");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn local_lib_dir(start: &Path) -> Option<PathBuf> {
    find_project_scrippo_dir(start).map(|d| d.join("lib"))
}

/// Project command entry: `.scrippo` file (legacy) or `.scrippo/commands.scrippo`.
pub fn project_commands_file(dir: &Path) -> Option<PathBuf> {
    let dot = dir.join(".scrippo");
    if dot.is_file() {
        return Some(dot);
    }
    if dot.is_dir() {
        let commands = dot.join("commands.scrippo");
        if commands.is_file() {
            return Some(commands);
        }
    }
    None
}

pub fn package_file_name(name: &str) -> String {
    let stem = strip_scrippo_ext(name);
    format!("{stem}.scrippo")
}

pub fn strip_scrippo_ext(name: &str) -> &str {
    name.strip_suffix(".scrippo").unwrap_or(name)
}

/// Bare package name: `mylib` (not `./x`, `a/b`, or `file.scrippo`).
pub fn is_package_name(pattern: &str) -> bool {
    if pattern.is_empty() || pattern.contains('/') || pattern.contains('.') {
        return false;
    }
    pattern
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn resolve_package(pattern: &str, from: &Path) -> Option<PathBuf> {
    if !is_package_name(pattern) {
        return None;
    }
    let file = package_file_name(pattern);
    if let Some(lib) = local_lib_dir(from) {
        for path in [lib.join(&file), lib.join(format!("{pattern}.bash"))] {
            if path.is_file() {
                return Some(path);
            }
        }
    }
    if let Ok(lib) = global_lib_dir() {
        for path in [lib.join(&file), lib.join(format!("{pattern}.bash"))] {
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// Resolve a script argument to a path. Tries the given path, then global scripts.
pub fn resolve_script_arg(arg: &Path) -> Result<PathBuf> {
    if arg.is_file() {
        return Ok(arg.to_path_buf());
    }
    if arg.is_dir() {
        let commands = arg.join("commands.scrippo");
        if commands.is_file() {
            return Ok(commands);
        }
        bail!("not a scrippo script: {}", arg.display());
    }

    let name = arg
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() || arg.components().count() != 1 {
        bail!("script not found: {}", arg.display());
    }

    let scripts = global_scripts_dir()?;
    let with_ext = package_file_name(&name);
    let stem = strip_scrippo_ext(&name);

    for candidate in [
        scripts.join(&with_ext),
        scripts.join(stem),
        scripts.join(format!("{stem}.scrippo")),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    bail!("script not found: {}", arg.display());
}

pub fn validate_name(name: &str) -> Result<String> {
    let stem = strip_scrippo_ext(name);
    if stem.is_empty() {
        bail!("name must not be empty");
    }
    if !stem
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        || !(stem.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_'))
    {
        bail!("invalid name '{stem}' (use letters, digits, _ or -; start with a letter or _)");
    }
    Ok(stem.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewKind {
    Package,
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewScope {
    Local,
    Global,
}

/// Directory for packages or scripts in the given scope.
pub fn scope_dir(kind: NewKind, scope: NewScope) -> Result<PathBuf> {
    Ok(match (kind, scope) {
        (NewKind::Package, NewScope::Local) => {
            let cwd = std::env::current_dir().context("failed to get current directory")?;
            let scrippo_dir = match find_project_scrippo_dir(&cwd) {
                Some(d) => d,
                None => {
                    let file = cwd.join(".scrippo");
                    if file.is_file() {
                        bail!(
                            "{} is a file; move it to .scrippo/commands.scrippo to use local packages",
                            file.display()
                        );
                    }
                    cwd.join(".scrippo")
                }
            };
            scrippo_dir.join("lib")
        }
        (NewKind::Package, NewScope::Global) => global_lib_dir()?,
        (NewKind::Script, NewScope::Global) => global_scripts_dir()?,
        (NewKind::Script, NewScope::Local) => {
            std::env::current_dir().context("failed to get current directory")?
        }
    })
}

pub fn create_new(kind: NewKind, scope: NewScope, name: &str) -> Result<PathBuf> {
    let stem = validate_name(name)?;
    let path = scope_dir(kind, scope)?.join(format!("{stem}.scrippo"));

    if path.exists() {
        bail!("already exists: {}", path.display());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let body = match kind {
        NewKind::Package => format!("# {stem} package\n"),
        NewKind::Script => format!("#! /usr/bin/env scrippo\n.main()\n\t-> \"hello from {stem}\"\n"),
    };
    std::fs::write(&path, body).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

/// Sorted unique entry names (file stems) in the scope directory.
pub fn list_entries(kind: NewKind, scope: NewScope) -> Result<Vec<String>> {
    let dir = scope_dir(kind, scope)?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = std::collections::BTreeSet::new();
    for item in std::fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let item = item?;
        let path = item.path();
        if !path.is_file() && !path.is_symlink() {
            continue;
        }
        let name = item.file_name();
        let name = name.to_string_lossy();
        let stem = name
            .strip_suffix(".scrippo")
            .or_else(|| name.strip_suffix(".bash"))
            .unwrap_or("");
        if stem.is_empty() {
            continue;
        }
        if validate_name(stem).is_ok() {
            names.insert(stem.to_string());
        }
    }
    Ok(names.into_iter().collect())
}

/// Remove a named package or script from the scope directory.
/// Deletes both `.scrippo` and `.bash` public links when present.
pub fn remove_entry(kind: NewKind, scope: NewScope, name: &str) -> Result<Vec<PathBuf>> {
    let stem = validate_name(name)?;
    let dir = scope_dir(kind, scope)?;
    let mut removed = Vec::new();
    for ext in ["scrippo", "bash"] {
        let path = dir.join(format!("{stem}.{ext}"));
        match std::fs::symlink_metadata(&path) {
            Ok(_) => {
                std::fs::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
                removed.push(path);
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("failed to stat {}", path.display()));
            }
        }
    }
    if removed.is_empty() {
        let kind = match kind {
            NewKind::Package => "package",
            NewKind::Script => "script",
        };
        bail!("{kind} not found: {stem}");
    }
    Ok(removed)
}
