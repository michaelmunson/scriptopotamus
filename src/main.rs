use anyhow::{Context, Result, bail};
use clap::Parser;
use scriptopotamus::compile_file;
use scriptopotamus::{config, paths, shell};
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

    /// Emit shell init for autoload (eval from your shell rc)
    #[arg(long = "init")]
    init: bool,

    /// Emit aliases/completions for the current directory (used by --init)
    #[arg(long = "env")]
    env: bool,

    /// Emit completion candidates for a scrippo file
    #[arg(long = "complete", value_name = "FILE")]
    complete: Option<PathBuf>,

    /// Compile to stdout (do not execute)
    #[arg(short = 'c', long = "compile")]
    compile_only: bool,

    /// Compile and run bash -n (syntax check)
    #[arg(long = "noexec")]
    noexec: bool,

    /// Script file, or completion words
    #[arg(value_name = "FILE", trailing_var_arg = true)]
    input: Vec<OsString>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(pairs) = cli.configure {
        return config::apply_configure(&pairs);
    }
    if cli.init {
        return shell::emit_init();
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
        .context("usage: scrippo [-c|--noexec] <file> [args...]")?;

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
