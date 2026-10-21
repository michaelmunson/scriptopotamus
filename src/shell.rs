use crate::ast::{Param, Program, Stmt};
use crate::config::Config;
use crate::macros;
use crate::parse;
use crate::paths;
use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct AliasEntry {
    name: String,
    file: PathBuf,
    prefix: Vec<String>,
}

pub fn emit_hook() -> Result<()> {
    // Detect at source/eval time: BASH_VERSION/ZSH_VERSION are not exported to `scrippo`.
    print!("{HOOK_UNIVERSAL}");
    Ok(())
}

pub fn emit_env() -> Result<()> {
    let cfg = Config::load()?;
    let shell = detect_shell();
    match shell {
        Shell::Bash => print!("{CLEAR_BASH}"),
        Shell::Zsh => {
            // compinit must run before any compdef (including clear).
            if cfg.autocomplete {
                print!("{COMP_INIT_ZSH}");
            }
            print!("{CLEAR_ZSH}");
        }
    }
    if !cfg.autoload {
        return Ok(());
    }
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let aliases = discover_aliases(&cwd)?;
    if aliases.is_empty() {
        return Ok(());
    }
    for entry in &aliases {
        let file = shell_quote(&entry.file.display().to_string());
        let mut cmd = format!("scrippo {file}");
        for part in &entry.prefix {
            cmd.push(' ');
            cmd.push_str(&shell_quote(part));
        }
        println!(
            "alias {}={}",
            shell_quote(&entry.name),
            shell_quote(&cmd)
        );
        println!("_SCRIPPO_MANAGED+=({})", shell_quote(&entry.name));
        println!(
            "_SCRIPPO_FILES[{}]={}",
            shell_quote(&entry.name),
            file
        );
        let prefix = entry
            .prefix
            .iter()
            .map(|s| shell_quote(s))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "_SCRIPPO_PREFIX[{}]={}",
            shell_quote(&entry.name),
            shell_quote(&prefix)
        );
    }
    if cfg.autocomplete {
        match shell {
            Shell::Bash => {
                println!("{COMP_FN_BASH}");
                for entry in &aliases {
                    println!(
                        "complete -o default -F _scrippo_comp {}",
                        shell_quote(&entry.name)
                    );
                }
            }
            Shell::Zsh => {
                println!("{COMP_FN_ZSH}");
                for entry in &aliases {
                    println!(
                        "compdef -a _scrippo_comp {}",
                        shell_quote(&entry.name)
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn emit_completions(file: &Path, words: &[String]) -> Result<()> {
    let program = match program_for_completion(file) {
        Some(p) => p,
        None => return Ok(()),
    };
    for c in complete_candidates(&program, words) {
        println!("{c}");
    }
    Ok(())
}

#[doc(hidden)]
pub fn completion_candidates_for_test(program: &Program, words: &[String]) -> Vec<String> {
    complete_candidates(program, words)
}

#[doc(hidden)]
pub fn completion_candidates_from_file(file: &Path, words: &[String]) -> Vec<String> {
    match program_for_completion(file) {
        Some(program) => complete_candidates(&program, words),
        None => Vec::new(),
    }
}

/// Parse a scrippo file and expand `@import`s so completion sees imported commands.
fn program_for_completion(file: &Path) -> Option<Program> {
    let canonical = file.canonicalize().ok()?;
    let file = canonical.as_path();
    let program = parse::parse_file(file).ok()?;
    match macros::expand_program(file, program.clone()) {
        Ok(expanded) => Some(expanded.program),
        Err(_) => Some(program),
    }
}

fn discover_aliases(dir: &Path) -> Result<Vec<AliasEntry>> {
    let mut entries = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    // Visit nearest directories first so local aliases shadow inherited ones.
    for ancestor in dir.ancestors() {
        discover_directory_aliases(ancestor, &mut entries, &mut names)?;
    }

    if let Ok(scripts_dir) = paths::global_scripts_dir() {
        if scripts_dir.is_dir() {
            let mut global = Vec::new();
            if let Ok(read) = fs::read_dir(&scripts_dir) {
                for item in read.flatten() {
                    let path = item.path();
                    if path.is_file() {
                        let name = item.file_name();
                        let name = name.to_string_lossy();
                        if name.ends_with(".scrippo") || has_scrippo_shebang(&path).unwrap_or(false)
                        {
                            global.push(path);
                        }
                    }
                }
            }
            global.sort();
            for path in global {
                push_script_alias(&mut entries, &mut names, path, Vec::new());
            }
        }
    }

    Ok(entries)
}

fn discover_directory_aliases(
    dir: &Path,
    entries: &mut Vec<AliasEntry>,
    names: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    let read = fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?;
    let mut scripts = Vec::new();

    for item in read {
        let item = item?;
        let path = item.path();
        if !path.is_file() {
            continue;
        }
        let name = item.file_name();
        let name = name.to_string_lossy();
        if name.as_ref() == ".scrippo" || name.starts_with('.') {
            continue;
        }
        if name.ends_with(".scrippo") || has_scrippo_shebang(&path).unwrap_or(false) {
            scripts.push(path);
        }
    }

    scripts.sort();
    for path in scripts {
        push_script_alias(entries, names, path, Vec::new());
    }

    if let Some(path) = paths::project_commands_file(dir) {
        let file = path.canonicalize().unwrap_or_else(|_| path.clone());
        if let Ok(program) = parse::parse_file(&path) {
            for cmd in top_level_commands(&program) {
                if !is_valid_alias_name(&cmd) {
                    continue;
                }
                if !names.insert(cmd.clone()) {
                    continue;
                }
                entries.push(AliasEntry {
                    name: cmd.clone(),
                    file: file.clone(),
                    prefix: vec![cmd],
                });
            }
        }
    }

    Ok(())
}

fn push_script_alias(
    entries: &mut Vec<AliasEntry>,
    names: &mut std::collections::BTreeSet<String>,
    path: PathBuf,
    prefix: Vec<String>,
) {
    let stem = alias_name_for(&path);
    if stem.is_empty() || !is_valid_alias_name(&stem) {
        return;
    }
    if !names.insert(stem.clone()) {
        return;
    }
    entries.push(AliasEntry {
        name: stem,
        file: path.canonicalize().unwrap_or(path),
        prefix,
    });
}

fn alias_name_for(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if let Some(stem) = name.strip_suffix(".scrippo") {
        stem.to_string()
    } else {
        name.to_string()
    }
}

fn has_scrippo_shebang(path: &Path) -> Result<bool> {
    let mut f = fs::File::open(path)?;
    let mut buf = [0u8; 256];
    let n = f.read(&mut buf)?;
    let text = String::from_utf8_lossy(&buf[..n]);
    let first = text.lines().next().unwrap_or("");
    Ok(first.starts_with("#!") && first.contains("scrippo"))
}

fn is_valid_alias_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn top_level_commands(program: &Program) -> Vec<String> {
    program
        .body
        .iter()
        .filter_map(|s| match s {
            Stmt::Function {
                name,
                is_command: true,
                ..
            } if !name.is_empty() => Some(name.clone()),
            _ => None,
        })
        .collect()
}

fn complete_candidates(program: &Program, words: &[String]) -> Vec<String> {
    let mut body = program.body.as_slice();
    let mut params: &[Param] = &[];
    let mut idx = 0;

    while idx < words.len() {
        let word = &words[idx];
        if word.starts_with('-') {
            break;
        }
        match find_command(body, word) {
            Some(Stmt::Function {
                params: p,
                body: next,
                ..
            }) => {
                params = p.as_slice();
                body = next.as_slice();
                idx += 1;
            }
            _ => break,
        }
    }

    let mut out = Vec::new();
    for stmt in body {
        if let Stmt::Function {
            name,
            is_command: true,
            ..
        } = stmt
        {
            if !name.is_empty() {
                out.push(name.clone());
            }
        }
    }
    collect_flags(params, &mut out);
    out.sort();
    out.dedup();
    out
}

fn find_command<'a>(body: &'a [Stmt], name: &str) -> Option<&'a Stmt> {
    body.iter().find(|s| {
        matches!(
            s,
            Stmt::Function {
                name: n,
                is_command: true,
                ..
            } if n == name
        )
    })
}

fn collect_flags(params: &[Param], out: &mut Vec<String>) {
    for param in params {
        match param {
            Param::Opt { flags, ty, .. } => {
                for f in flags {
                    out.push(f.clone());
                }
                if let Some(lits) = &ty.literals {
                    out.extend(lits.iter().cloned());
                }
            }
            Param::Arg { ty, .. } | Param::Rest { ty, .. } => {
                if let Some(lits) = &ty.literals {
                    out.extend(lits.iter().cloned());
                }
            }
            Param::Group { params, .. } => collect_flags(params, out),
            Param::Exclusive { options } => collect_flags(options, out),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shell {
    Bash,
    Zsh,
}

fn detect_shell() -> Shell {
    if let Ok(s) = std::env::var("SCRIPPO_SHELL") {
        match s.to_ascii_lowercase().as_str() {
            "zsh" => return Shell::Zsh,
            "bash" => return Shell::Bash,
            _ => {}
        }
    }
    if std::env::var_os("ZSH_VERSION").is_some() {
        return Shell::Zsh;
    }
    if std::env::var_os("BASH_VERSION").is_some() {
        return Shell::Bash;
    }
    if let Some(shell) = parent_shell() {
        return shell;
    }
    match std::env::var("SHELL") {
        Ok(s) if s.contains("zsh") => Shell::Zsh,
        Ok(s) if s.contains("bash") => Shell::Bash,
        _ => Shell::Bash,
    }
}

fn parent_shell() -> Option<Shell> {
    let ppid = std::os::unix::process::parent_id();
    if ppid == 0 {
        return None;
    }
    let name = parent_comm(ppid)?;
    let base = name.rsplit('/').next().unwrap_or(&name);
    if base.contains("zsh") {
        Some(Shell::Zsh)
    } else if base.contains("bash") {
        Some(Shell::Bash)
    } else {
        None
    }
}

fn parent_comm(ppid: u32) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let text = fs::read_to_string(format!("/proc/{ppid}/comm")).ok()?;
        return Some(text.trim().to_string());
    }
    #[cfg(not(target_os = "linux"))]
    {
        let output = std::process::Command::new("ps")
            .args(["-p", &ppid.to_string(), "-o", "comm="])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    }
}

fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':'))
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

const CLEAR_BASH: &str = r#"if (( ${#_SCRIPPO_MANAGED[@]} > 0 )); then
  for _scrippo_name in "${_SCRIPPO_MANAGED[@]}"; do
    unalias "$_scrippo_name" 2>/dev/null || true
    complete -r "$_scrippo_name" 2>/dev/null || true
  done
fi
_SCRIPPO_MANAGED=()
declare -gA _SCRIPPO_FILES=()
declare -gA _SCRIPPO_PREFIX=()
"#;

const CLEAR_ZSH: &str = r#"if (( ${#_SCRIPPO_MANAGED[@]} > 0 )); then
  for _scrippo_name in "${_SCRIPPO_MANAGED[@]}"; do
    unalias "$_scrippo_name" 2>/dev/null || true
    compdef -d "$_scrippo_name" 2>/dev/null || true
  done
fi
_SCRIPPO_MANAGED=()
typeset -gA _SCRIPPO_FILES
typeset -gA _SCRIPPO_PREFIX
_SCRIPPO_FILES=()
_SCRIPPO_PREFIX=()
"#;

const HOOK_UNIVERSAL: &str = r#"if [ -n "${ZSH_VERSION-}" ]; then
  setopt COMPLETE_ALIASES
  _scrippo_hook() {
    eval "$(SCRIPPO_SHELL=zsh scrippo --env)"
  }
  autoload -Uz add-zsh-hook
  add-zsh-hook chpwd _scrippo_hook
  _scrippo_hook
elif [ -n "${BASH_VERSION-}" ]; then
  _scrippo_hook() {
    [[ $PWD == ${_SCRIPPO_LAST_PWD-} ]] && return
    _SCRIPPO_LAST_PWD=$PWD
    eval "$(SCRIPPO_SHELL=bash scrippo --env)"
  }
  if [[ ${PROMPT_COMMAND-} != *_scrippo_hook* ]]; then
    PROMPT_COMMAND="_scrippo_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
  fi
  _scrippo_hook
else
  echo "scrippo --hook: unsupported shell (need bash or zsh)" >&2
fi
"#;

const COMP_INIT_ZSH: &str = r#"setopt COMPLETE_ALIASES
autoload -Uz compinit
(( $+functions[compdef] )) || compinit -C
"#;

const COMP_FN_BASH: &str = r#"_scrippo_comp() {
  local cur=${COMP_WORDS[COMP_CWORD]}
  local cmd=${COMP_WORDS[0]}
  local file=${_SCRIPPO_FILES[$cmd]}
  local prefix=${_SCRIPPO_PREFIX[$cmd]}
  local -a path_words=()
  local -a typed=()
  [[ -n $prefix ]] && read -r -a path_words <<<"$prefix"
  if (( COMP_CWORD > 1 )); then
    typed=("${COMP_WORDS[@]:1:$((COMP_CWORD - 1))}")
  fi
  path_words+=("${typed[@]}")
  local -a candidates
  if (( ${#path_words[@]} > 0 )); then
    mapfile -t candidates < <(scrippo --complete "$file" -- "${path_words[@]}")
  else
    mapfile -t candidates < <(scrippo --complete "$file" --)
  fi
  COMPREPLY=($(compgen -W "${candidates[*]}" -- "$cur"))
}
"#;

const COMP_FN_ZSH: &str = r#"_scrippo_comp() {
  local cmd=${words[1]}
  local file=${_SCRIPPO_FILES[$cmd]}
  [[ -n $file ]] || return 1
  local prefix=${_SCRIPPO_PREFIX[$cmd]}
  local -a path_words=()
  [[ -n $prefix ]] && path_words=(${=prefix})
  if (( CURRENT > 2 )); then
    path_words+=("${(@)words[2,CURRENT-1]}")
  fi
  local -a candidates
  if (( ${#path_words[@]} > 0 )); then
    candidates=(${(f)"$(scrippo --complete "$file" -- "${path_words[@]}")"})
  else
    candidates=(${(f)"$(scrippo --complete "$file" --)"})
  fi
  (( ${#candidates[@]} )) || return 1
  compadd -a -- candidates
}
"#;
