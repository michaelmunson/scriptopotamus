//! Tagged Git installs into the existing package and script search paths.
use crate::ast::Stmt;
use crate::macros::info::{PackageInfo, Version};
use crate::paths::{self, NewKind};
use anyhow::{Context, Result, bail};
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

pub struct InstallOptions {
    pub repository: String,
    pub kind: NewKind,
    pub global: bool,
    pub destination: Option<PathBuf>,
    pub tag: Option<String>,
}

pub struct InstallResult {
    pub tag: String,
    pub commit: String,
    pub files: Vec<PathBuf>,
}

pub fn install(options: &InstallOptions) -> Result<InstallResult> {
    if options.global && options.destination.is_some() {
        bail!("--global cannot be combined with a destination directory");
    }
    if let Some(tag) = &options.tag {
        version_tag(tag).context("--tag must be a stable version such as 1.2.3 or v1.2.3")?;
    }
    let repo_name = repository_name(&options.repository)?;
    let destination = destination(options)?;
    fs::create_dir_all(&destination)?;
    let destination = destination.canonicalize()?;
    let scratch = tempfile::Builder::new()
        .prefix(".scrippo-install-")
        .tempdir_in(&destination)?;
    let git_dir = scratch.path().join("git");
    checked(
        Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "clone",
                "--quiet",
                "--bare",
                "--",
            ])
            .arg(&options.repository)
            .arg(&git_dir),
    )?;
    let tag = match &options.tag {
        Some(tag) => tag.clone(),
        None => {
            let output = git(&git_dir, &["tag", "--list"])?;
            String::from_utf8(output.stdout)?
                .lines()
                .filter_map(|tag| version_tag(tag).map(|version| (version, tag.to_string())))
                .max()
                .map(|(_, tag)| tag)
                .context("repository has no stable version tags (expected 1.2.3 or v1.2.3)")?
        }
    };
    let version = version_tag(&tag).unwrap();
    let revision = format!("refs/tags/{tag}^{{commit}}");
    let commit = String::from_utf8(git(&git_dir, &["rev-parse", "--verify", &revision])?.stdout)?
        .trim()
        .to_string();
    let staged = scratch.path().join("content");
    let source = staged.join("source");
    fs::create_dir_all(&source)?;
    let entries = snapshot(&git_dir, &commit, &source)?;
    let kind = match options.kind {
        NewKind::Package => "packages",
        NewKind::Script => "scripts",
    };
    let selected: Vec<_> = entries
        .iter()
        .filter(|path| {
            path.starts_with(kind) && path.extension().is_some_and(|ext| ext == "scrippo")
        })
        .collect();
    if selected.is_empty() {
        bail!("repository has no .scrippo files in {kind}/");
    }
    let mut names = BTreeSet::new();
    let mut artifacts = Vec::new();
    let mut metadata = Vec::new();
    for relative in selected {
        let file = source.join(relative);
        let info = entry_info(&file, &version)?;
        if !names.insert(info.name.clone()) {
            bail!("duplicate {kind} entry name: {}", info.name);
        }
        let artifact = match options.kind {
            NewKind::Package => {
                let bash = crate::compile_package(&file, &source)?;
                let output = staged.join("compiled").join(format!("{}.bash", info.name));
                fs::create_dir_all(output.parent().unwrap())?;
                fs::write(&output, bash)?;
                checked(Command::new("bash").arg("-n").arg(&output))?;
                output.strip_prefix(&staged)?.to_path_buf()
            }
            NewKind::Script => PathBuf::from("source").join(relative),
        };
        let extension = match options.kind {
            NewKind::Package => "bash",
            NewKind::Script => "scrippo",
        };
        let filename = format!("{}.{extension}", info.name);
        artifacts.push((filename, artifact));
        metadata
            .push(json!({"name": info.name, "description": info.description, "source": relative}));
    }
    fs::write(
        staged.join("install.json"),
        serde_json::to_vec_pretty(&json!({
            "repository": options.repository, "tag": tag, "commit": commit, "kind": kind, "entries": metadata,
        }))?,
    )?;

    // Cooperating installers serialize publication; each public link is also
    // created with no-clobber semantics. Roll back only paths this run created.
    let lock = destination.join(".scrippo-install.lock");
    fs::create_dir(&lock).context("another install is publishing here (or .scrippo-install.lock remains from an interrupted process)")?;
    let _lock = LockDir(lock);
    for name in &names {
        for ext in ["scrippo", "bash"] {
            let path = destination.join(format!("{name}.{ext}"));
            if fs::symlink_metadata(&path).is_ok() {
                bail!("destination already exists: {}", path.display());
            }
        }
    }
    let repo_dir = destination
        .join(".scrippo-installed")
        .join(kind)
        .join(repo_name);
    fs::create_dir_all(repo_dir.parent().unwrap())?;
    fs::create_dir(&repo_dir).context("repository is already installed at this destination")?;
    let mut publication = Publication {
        files: Vec::new(),
        repository: repo_dir.clone(),
        committed: false,
    };
    fs::rename(staged, repo_dir.join("content"))?;
    for (name, relative) in artifacts {
        let stored = repo_dir.join("content").join(relative);
        let public = destination.join(name);
        match options.kind {
            NewKind::Package => fs::hard_link(&stored, &public)?,
            NewKind::Script => script_link(stored.strip_prefix(&destination)?, &public)?,
        }
        publication.files.push(public);
    }
    publication.committed = true;
    Ok(InstallResult {
        tag,
        commit,
        files: publication.files.clone(),
    })
}

fn destination(options: &InstallOptions) -> Result<PathBuf> {
    if let Some(path) = &options.destination {
        return Ok(path.clone());
    }
    if options.global {
        return match options.kind {
            NewKind::Package => paths::global_lib_dir(),
            NewKind::Script => paths::global_scripts_dir(),
        };
    }
    let cwd = std::env::current_dir()?;
    Ok(match options.kind {
        NewKind::Package => paths::find_project_scrippo_dir(&cwd)
            .unwrap_or_else(|| cwd.join(".scrippo"))
            .join("lib"),
        NewKind::Script => cwd,
    })
}

fn version_tag(tag: &str) -> Option<Version> {
    Version::parse(tag.strip_prefix('v').unwrap_or(tag))
}

fn repository_name(repository: &str) -> Result<&str> {
    let name = repository
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("");
    let name = name.strip_suffix(".git").unwrap_or(name);
    if name.is_empty()
        || !name.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    {
        bail!("repository must have a simple basename");
    }
    Ok(name)
}

fn entry_info(file: &Path, version: &Version) -> Result<PackageInfo> {
    let program = crate::parse::parse_file(file)?;
    let declarations: Vec<_> = program
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::MacroCall { name, args, line } if name == "info" => Some((args, line)),
            _ => None,
        })
        .collect();
    if declarations.len() != 1 {
        bail!("{}: expected exactly one top-level @info", file.display());
    }
    let (args, line) = declarations[0];
    let info = PackageInfo::parse(file, *line, args)?;
    if file.file_stem().and_then(|s| s.to_str()) != Some(&info.name) {
        bail!("{}: @info name must match the filename", file.display());
    }
    if &info.version != version {
        bail!(
            "{}: @info version must match the selected tag",
            file.display()
        );
    }
    Ok(info)
}

/// Read blob objects directly, never checking out hooks, filters, or links.
fn snapshot(git_dir: &Path, commit: &str, destination: &Path) -> Result<Vec<PathBuf>> {
    let tree = git(git_dir, &["ls-tree", "-rz", "--full-tree", commit])?;
    let mut entries = Vec::new();
    for entry in tree
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let entry = std::str::from_utf8(entry).context("repository paths must be UTF-8")?;
        let (header, name) = entry.split_once('\t').context("invalid Git tree entry")?;
        let fields: Vec<_> = header.split_whitespace().collect();
        if fields.len() != 3 || !matches!(fields[0], "100644" | "100755") || fields[1] != "blob" {
            bail!("repository links and submodules are not supported");
        }
        let relative = Path::new(name);
        if !relative
            .components()
            .all(|part| matches!(part, Component::Normal(s) if s != ".git"))
        {
            bail!("invalid repository path: {name}");
        }
        let path = destination.join(relative);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(
            &path,
            git(git_dir, &["cat-file", "blob", fields[2]])?.stdout,
        )?;
        #[cfg(unix)]
        if fields[0] == "100755" {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        }
        entries.push(relative.to_path_buf());
    }
    Ok(entries)
}

fn git(directory: &Path, args: &[&str]) -> Result<Output> {
    checked(Command::new("git").arg("-C").arg(directory).args(args))
}

fn checked(command: &mut Command) -> Result<Output> {
    let output = command
        .output()
        .with_context(|| format!("failed to run {:?}", command.get_program()))?;
    if !output.status.success() {
        bail!(
            "{} failed: {}",
            command.get_program().to_string_lossy(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output)
}

#[cfg(unix)]
fn script_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
fn script_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(source, destination)
}

struct LockDir(PathBuf);
impl Drop for LockDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

struct Publication {
    files: Vec<PathBuf>,
    repository: PathBuf,
    committed: bool,
}
impl Drop for Publication {
    fn drop(&mut self) {
        if !self.committed {
            for file in &self.files {
                let _ = fs::remove_file(file);
            }
            let _ = fs::remove_dir_all(&self.repository);
        }
    }
}
