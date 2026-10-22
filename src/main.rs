use anyhow::{Context, Result, bail};
use clap::Parser;
use scriptopotamus::compile_file;
use scriptopotamus::{config, packages, paths, shell};
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
// The user-facing `scrippo` is cli/scrippo.scrippo compiled to bash, which validates
// its arguments and execs this binary (`scrippo-core`) with them unchanged.
#[command(name = "scrippo", version, about = "Scriptopotamus compiler")]
struct Cli {
    /// Configure scrippo settings (key=value). Prints current config when given alone.
    #[arg(long = "configure", num_args = 0.., value_name = "KEY=VALUE")]
    configure: Option<Vec<String>>,

    /// Create a new package or script
    #[arg(long = "new")]
    new: bool,

    /// Install a version-tagged Git repository
    #[arg(short = 'i', long = "install", conflicts_with_all = ["new", "configure", "hook", "env", "complete", "compile_only", "noexec"])]
    install: bool,

    /// Exact stable version tag to install (default: highest stable version)
    #[arg(long, requires = "install")]
    tag: Option<String>,

    /// Select packages (with --new or --install)
    #[arg(short = 'p', long = "package", conflicts_with = "script")]
    package: bool,

    /// Select scripts (with --new or --install)
    #[arg(short = 's', long = "script", conflicts_with = "package")]
    script: bool,

    /// Use the project / current directory (with --new or --install)
    #[arg(long = "local", conflicts_with = "global")]
    local: bool,

    /// Use ~/.scrippo, or SCRIPPO_HOME (with --new or --install)
    #[arg(short = 'g', long = "global", conflicts_with = "local")]
    global: bool,

    /// Emit shell hook for autoload (eval from your shell rc)
    #[arg(long = "hook")]
    hook: bool,

    /// Emit aliases/completions for the current directory (used by --hook)
    #[arg(long = "env")]
    env: bool,

    /// Emit completion candidates for a scrippo file
    #[arg(long = "complete", value_name = "FILE")]
    complete: Option<PathBuf>,

    /// Compile to stdout (do not execute)
    #[arg(short = 'c', long = "compile")]
    compile_only: bool,

    /// Compile and run bash -n (syntax check)
    #[arg(short = 'n', long = "noexec")]
    noexec: bool,

    /// Script file, name for --new, repository for --install, or completion words
    #[arg(value_name = "FILE", trailing_var_arg = true)]
    input: Vec<OsString>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.install {
        return run_install(&cli);
    }

    if let Some(pairs) = cli.configure {
        return config::apply_configure(&pairs);
    }
    if cli.new {
        return run_new(&cli);
    }
    if cli.hook {
        return shell::emit_hook();
    }
    if cli.env {
        return shell::emit_env();
    }
    if let Some(file) = cli.complete {
        let words: Vec<_> = cli
            .input
            .iter()
            .map(|word| word.to_string_lossy().into_owned())
            .collect();
        let words = if !words.is_empty() && words[0] == "--" {
            words[1..].to_vec()
        } else {
            words
        };
        return shell::emit_completions(&file, &words);
    }

    let file = cli
        .input
        .first()
        .map(PathBuf::from)
        .context("usage: scrippo [-c|-n] <file> [args...]")?;

    let file = paths::resolve_script_arg(&file)?;
    let bash = compile_file(&file).map_err(|e| anyhow::anyhow!("{e}"))?;

    if cli.compile_only {
        print!("{bash}");
        return Ok(());
    }

    let tmp = tempfile_path()?;
    {
        let mut f = std::fs::File::create(&tmp).context("failed to create temp script")?;
        f.write_all(bash.as_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = f.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&tmp, perms)?;
        }
    }

    let result = if cli.noexec {
        let status = Command::new("bash")
            .arg("-n")
            .arg(&tmp)
            .status()
            .context("failed to run bash -n")?;
        if status.success() {
            Ok(())
        } else {
            bail!("bash -n failed")
        }
    } else {
        let source = file.canonicalize().unwrap_or_else(|_| file.clone());
        let script_args = if cli.input.get(1).is_some_and(|arg| arg == "--") {
            &cli.input[2..]
        } else {
            &cli.input[1..]
        };
        let status = Command::new("bash")
            .env("_SCRIPPO_FILE", &source)
            .arg(&tmp)
            .args(script_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to execute compiled script")?;
        if status.success() {
            Ok(())
        } else {
            std::process::exit(status.code().unwrap_or(1));
        }
    };

    let _ = std::fs::remove_file(&tmp);
    result
}

fn run_install(cli: &Cli) -> Result<()> {
    let repository = cli
        .input
        .first()
        .context("--install requires a Git repository")?
        .to_str()
        .context("repository must be UTF-8")?
        .to_string();
    let kind = match (cli.package, cli.script) {
        (true, false) => paths::NewKind::Package,
        (false, true) => paths::NewKind::Script,
        _ => bail!("--install requires --package or --script"),
    };
    if cli.input.len() > 2
        || cli
            .input
            .get(1)
            .is_some_and(|arg| arg.to_string_lossy().starts_with('-'))
    {
        bail!(
            "usage: scrippo -i (-p|-s) [-g] [--tag VERSION] <repository> [directory]; put options before the repository"
        );
    }
    let result = packages::install(&packages::InstallOptions {
        repository,
        kind,
        global: cli.global,
        destination: cli.input.get(1).map(PathBuf::from),
        tag: cli.tag.clone(),
    })?;
    println!("installed {} ({})", result.tag, result.commit);
    for file in result.files {
        println!("  {}", file.display());
    }
    Ok(())
}

fn run_new(cli: &Cli) -> Result<()> {
    let name = cli
        .input
        .first()
        .map(|p| p.to_string_lossy().into_owned())
        .context("usage: scrippo --new [--package|--script] [--local|--global] <name>")?;

    let kind = match (cli.package, cli.script) {
        (true, false) => paths::NewKind::Package,
        (false, true) => paths::NewKind::Script,
        (false, false) => bail!("specify --package or --script"),
        (true, true) => unreachable!("clap conflicts_with"),
    };

    let scope = match (cli.local, cli.global) {
        (true, false) => paths::NewScope::Local,
        (false, true) => paths::NewScope::Global,
        (false, false) => match kind {
            paths::NewKind::Package => paths::NewScope::Local,
            paths::NewKind::Script => paths::NewScope::Global,
        },
        (true, true) => unreachable!("clap conflicts_with"),
    };

    let path = paths::create_new(kind, scope, &name)?;
    println!("created {}", path.display());
    Ok(())
}

fn tempfile_path() -> Result<PathBuf> {
    let mut path = std::env::temp_dir();
    let name = format!(
        "scrippo-{}-{}.bash",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    path.push(name);
    Ok(path)
}
