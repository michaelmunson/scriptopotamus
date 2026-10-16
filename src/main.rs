use anyhow::{bail, Context, Result};
use clap::Parser;
use scriptopotamus::compile_file;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(name = "scrippo", version, about = "Scriptopotamus compiler")]
struct Cli {
    /// Compile to stdout (do not execute)
    #[arg(short = 'c', long = "compile")]
    compile_only: bool,

    /// Compile and run bash -n (syntax check)
    #[arg(short = 'n', long = "noexec")]
    noexec: bool,

    /// Script file to compile
    file: Option<PathBuf>,

    /// Arguments passed to the compiled script when executing
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file = cli
        .file
        .clone()
        .context("usage: scrippo <file> [-c|-n] [-- args...]")?;

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
        let status = Command::new("bash")
            .arg(&tmp)
            .args(&cli.args)
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
