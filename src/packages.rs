//! Tagged Git (or plain directory) installs into package and script search paths.
use crate::ast::{Program, Stmt};
use crate::macros::info::{FileKind, PackageInfo, Version};
use crate::paths::{self, NewKind};
use anyhow::{Context, Result, bail};
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
    let staged = scratch.path().join("content");
    let source = staged.join("source");
    fs::create_dir_all(&source)?;

    let (tag, commit, entries) = stage_source(options, scratch.path(), &source)?;
    let version = version_tag(&tag).unwrap();
    let kind = match options.kind {
        NewKind::Package => "packages",
        NewKind::Script => "scripts",
    };
    let selected: Vec<_> = entries
        .iter()
        .filter(|path| {
            path.starts_with(kind) && path.extension().is_some_and(|ext| ext == "scrippo")
        })
        .cloned()
        .collect();
    if selected.is_empty() {
        bail!("repository has no .scrippo files in {kind}/");
    }

    run_install_hook(&source, "before")?;

    let mut names = BTreeSet::new();
    let mut artifacts = Vec::new();
    let mut metadata = Vec::new();
    for relative in &selected {
        let file = source.join(relative);
        let info = entry_info(&file, &version)?;
        let expected_kind = match options.kind {
            NewKind::Package => FileKind::Package,
            NewKind::Script => FileKind::Script,
        };
        if info.metadata.kind.is_some_and(|kind| kind != expected_kind) {
            bail!(
                "{}: @info kind must match the selected {} directory",
                file.display(),
                kind
            );
        }
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
        metadata.push(json!({
            "name": info.name, "version": info.version.0.iter().map(u64::to_string).collect::<Vec<_>>().join("."),
            "description": info.description, "author": info.metadata.author,
            "license": info.metadata.license, "repository": info.metadata.repository,
            "kind": expected_kind.as_str(), "autoload": info.metadata.autoload, "source": relative,
        }));
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
    fs::rename(&staged, repo_dir.join("content"))?;
    let published_source = repo_dir.join("content/source");
    for (name, relative) in artifacts {
        let stored = repo_dir.join("content").join(relative);
        let public = destination.join(name);
        match options.kind {
            NewKind::Package => fs::hard_link(&stored, &public)?,
            NewKind::Script => script_link(stored.strip_prefix(&destination)?, &public)?,
        }
        publication.files.push(public);
    }

    run_install_hook(&published_source, "after")?;
    publication.committed = true;
    Ok(InstallResult {
        tag,
        commit,
        files: publication.files.clone(),
    })
}

fn stage_source(
    options: &InstallOptions,
    scratch: &Path,
    source: &Path,
) -> Result<(String, String, Vec<PathBuf>)> {
    let path = Path::new(&options.repository);
    if path.is_dir() && !path.join(".git").exists() {
        let entries = snapshot_directory(path, source)?;
        let tag = directory_tag(options, source, &entries)?;
        return Ok((tag, "local".into(), entries));
    }

    let git_dir = scratch.join("git");
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
    let revision = format!("refs/tags/{tag}^{{commit}}");
    let commit = String::from_utf8(git(&git_dir, &["rev-parse", "--verify", &revision])?.stdout)?
        .trim()
        .to_string();
    let entries = snapshot(&git_dir, &commit, source)?;
    Ok((tag, commit, entries))
}

fn directory_tag(
    options: &InstallOptions,
    source: &Path,
    entries: &[PathBuf],
) -> Result<String> {
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
    if let Some(tag) = &options.tag {
        return Ok(tag.clone());
    }
    let mut found: Option<Version> = None;
    for relative in selected {
        let info = read_entry_info(&source.join(relative))?;
        match &found {
            None => found = Some(info.version.clone()),
            Some(version) if version != &info.version => {
                bail!("directory entries must share one @info version when --tag is omitted");
            }
            _ => {}
        }
    }
    let version = found.unwrap();
    Ok(format!(
        "{}.{}.{}",
        version.0[0], version.0[1], version.0[2]
    ))
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

fn read_entry_info(file: &Path) -> Result<PackageInfo> {
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
    Ok(info)
}

fn entry_info(file: &Path, version: &Version) -> Result<PackageInfo> {
    let info = read_entry_info(file)?;
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

fn snapshot_directory(root: &Path, destination: &Path) -> Result<Vec<PathBuf>> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", root.display()))?;
    let mut entries = Vec::new();
    fn walk(root: &Path, dir: &Path, destination: &Path, entries: &mut Vec<PathBuf>) -> Result<()> {
        for item in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let item = item?;
            let name = item.file_name();
            if name == ".git" {
                continue;
            }
            let path = item.path();
            let meta = fs::symlink_metadata(&path)?;
            if meta.file_type().is_symlink() {
                bail!("repository links and submodules are not supported");
            }
            let relative = path
                .strip_prefix(root)
                .with_context(|| format!("path escaped install root: {}", path.display()))?;
            if !relative
                .components()
                .all(|part| matches!(part, Component::Normal(s) if s != ".git"))
            {
                bail!("invalid repository path: {}", relative.display());
            }
            if meta.is_dir() {
                walk(root, &path, destination, entries)?;
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            let target = destination.join(relative);
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(&path, &target)
                .with_context(|| format!("failed to copy {}", path.display()))?;
            entries.push(relative.to_path_buf());
        }
        Ok(())
    }
    walk(&root, &root, destination, &mut entries)?;
    Ok(entries)
}

fn run_install_hook(source: &Path, phase: &str) -> Result<()> {
    let hook = source.join(".install.scrippo");
    if !hook.is_file() {
        return Ok(());
    }
    let mut program = crate::parse::parse_file(&hook).map_err(|e| anyhow::anyhow!("{e}"))?;
    ensure_install_hooks(&mut program);
    let bash = crate::compile_program(&hook, program).map_err(|e| anyhow::anyhow!("{e}"))?;
    let tmp = tempfile::Builder::new()
        .prefix(&format!("scrippo-install-{phase}-"))
        .suffix(".bash")
        .tempfile()
        .context("failed to create install hook temp file")?;
    fs::write(tmp.path(), bash.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o755))?;
    }
    let status = Command::new("bash")
        .env("_SCRIPPO_FILE", &hook)
        .env("SCRIPPO_INSTALL_PHASE", phase)
        .current_dir(source)
        .arg(tmp.path())
        .arg(phase)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run .install.scrippo {phase}"))?;
    if !status.success() {
        bail!(
            ".install.scrippo {phase} failed{}",
            status
                .code()
                .map(|code| format!(" (exit {code})"))
                .unwrap_or_default()
        );
    }
    Ok(())
}

fn ensure_install_hooks(program: &mut Program) {
    for name in ["before", "after"] {
        let present = program.body.iter().any(|stmt| {
            matches!(
                stmt,
                Stmt::Function {
                    name: n,
                    is_command: true,
                    ..
                } if n == name
            )
        });
        if !present {
            program.body.push(Stmt::Function {
                name: name.into(),
                is_command: true,
                docs: Vec::new(),
                params: Vec::new(),
                body: Vec::new(),
                line: 0,
            });
        }
    }
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
