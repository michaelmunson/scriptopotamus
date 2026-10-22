use scriptopotamus::shell::{CLI_SELF, CLI_SOURCE, completion_candidates_from_file};
use scriptopotamus::compile_source;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CORE: &str = env!("CARGO_BIN_EXE_scrippo-core");

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scrippo-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The compiled `scrippo` wrapper, driving the freshly built core binary.
fn wrapper(dir: &Path) -> PathBuf {
    let bash = compile_source(Path::new("scrippo.scrippo"), CLI_SOURCE).unwrap();
    let path = dir.join("scrippo");
    std::fs::write(&path, bash).unwrap();
    path
}

fn scrippo(dir: &Path, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(wrapper(dir))
        .args(args)
        .env("SCRIPPO_CORE", CORE)
        .env("SCRIPPO_HOME", dir.join("home"))
        .env("NO_COLOR", "1")
        .current_dir(dir)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn complete(list: &[&str]) -> Vec<String> {
    completion_candidates_from_file(Path::new(CLI_SELF), &words(list))
}

#[test]
fn wrapper_compiles_to_valid_bash() {
    let dir = temp_dir("syntax");
    let path = wrapper(&dir);
    let out = Command::new("bash").arg("-n").arg(&path).output().unwrap();
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn usage_nests_options_under_their_mode() {
    let dir = temp_dir("usage");
    let out = scrippo(&dir, &["--help"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("Usage:\n  scrippo # "), "{text}");
    assert!(text.contains("[-h / --help] # Show this usage"), "{text}");
    assert!(text.contains("- [-c / --compile] # Compile to stdout"), "{text}");
    assert!(text.contains("- [-i / --install] # Install"), "{text}");
    assert!(text.contains("- [-n / --new]"), "{text}");
    assert!(text.contains("- [-ls / --list]"), "{text}");
    assert!(text.contains("- [-rm / --remove]"), "{text}");
    assert!(!text.contains("--local"), "{text}");
    // --tag nests beneath --install, and the shared options beneath the modes.
    let install = text.find("- [-i / --install]").unwrap();
    let tag = text.find("[--tag]<str>").unwrap();
    let package = text.find("- [-p / --package]").unwrap();
    assert!(install < tag && tag < package, "{text}");
    let indent = |needle: &str| {
        let line = text.lines().find(|l| l.contains(needle)).unwrap();
        line.len() - line.trim_start().len()
    };
    assert!(indent("[--tag]") > indent("- [-i / --install]"), "{text}");
    assert!(text.contains("[args]... # Passed through to the script"), "{text}");
}

#[test]
fn no_arguments_print_usage_and_fail() {
    let dir = temp_dir("noargs");
    let out = scrippo(&dir, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).starts_with("Usage:"), "{}", stdout(&out));
}

#[test]
fn modes_and_modifiers_are_exclusive() {
    let dir = temp_dir("exclusive");
    for (args, message) in [
        (vec!["-c", "--noexec", "x.scrippo"], "-c and --noexec are mutually exclusive"),
        (vec!["-c", "-n", "x.scrippo"], "-c and [-n"),
        (vec!["--new", "-p", "-s", "x"], "-p and -s are mutually exclusive"),
        (
            vec!["--new", "-i", "-p", "x"],
            "-n and [-i --tag] are mutually exclusive",
        ),
        (vec!["--new", "--tag", "v1", "-p", "x"], "-n and [-i --tag]"),
        (vec!["-n", "-ls", "-p"], "-n and -ls are mutually exclusive"),
        (vec!["-ls", "-rm", "-s"], "-ls and -rm are mutually exclusive"),
        (vec!["-c", "--hook"], "-c and --hook are mutually exclusive"),
        (vec!["--hook", "--env"], "--hook and --env are mutually exclusive"),
        (vec!["-c", "-p", "x.scrippo"], "-c and [-n"),
    ] {
        let out = scrippo(&dir, &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        let err = stderr(&out);
        assert!(err.contains(message), "{args:?}: {err}");
        assert!(err.contains("Usage:"), "{args:?}: {err}");
    }
}

#[test]
fn wrapper_delegates_to_core() {
    let dir = temp_dir("delegate");
    let out = scrippo(&dir, &["--version"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).starts_with("scrippo "), "{}", stdout(&out));

    let out = scrippo(&dir, &["-n", "-p", "helpers"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(dir.join(".scrippo/lib/helpers.scrippo").is_file());

    std::fs::write(dir.join("hi.scrippo"), ".greet(name)\n  -> \"hello $name\"\n").unwrap();
    let out = scrippo(&dir, &["hi.scrippo", "greet", "bob"]);
    assert_eq!(stdout(&out), "hello bob\n", "{}", stderr(&out));
    let out = scrippo(&dir, &["-c", "hi.scrippo"]);
    assert!(stdout(&out).contains("function greet"), "{}", stderr(&out));
    let out = scrippo(&dir, &["--noexec", "hi.scrippo"]);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn list_and_remove_scripts_and_packages() {
    let dir = temp_dir("list-rm");
    assert!(scrippo(&dir, &["-n", "-p", "helpers"]).status.success());
    assert!(scrippo(&dir, &["-n", "-s", "hello"]).status.success());
    assert!(scrippo(&dir, &["-n", "-s", "-g", "greet"]).status.success());
    assert!(scrippo(&dir, &["-n", "-p", "-g", "utils"]).status.success());

    let out = scrippo(&dir, &["-ls", "-p"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "helpers\n");

    let out = scrippo(&dir, &["-ls", "-s"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "hello\n");

    let out = scrippo(&dir, &["-ls", "-s", "-g"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "greet\n");

    let out = scrippo(&dir, &["-ls", "-p", "-g"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "utils\n");

    let out = scrippo(&dir, &["-rm", "-s", "hello"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!dir.join("hello.scrippo").exists());
    assert!(stdout(&out).contains("removed"), "{}", stdout(&out));

    let out = scrippo(&dir, &["-rm", "-p", "-g", "utils"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!dir.join("home/lib/utils.scrippo").exists());
}

#[test]
fn script_arguments_are_not_interpreted_by_the_wrapper() {
    let dir = temp_dir("passthrough");
    std::fs::write(
        dir.join("tool.scrippo"),
        ".run([-c] [-n] [-p] [-s] [--tag<str>])\n  -> \"c=$c n=$n p=$p s=$s tag=$tag\"\n",
    )
    .unwrap();
    let out = scrippo(&dir, &["tool.scrippo", "run", "-c", "-n", "-p", "-s", "--tag", "v1"]);
    assert_eq!(stdout(&out), "c=true n=true p=true s=true tag=v1\n", "{}", stderr(&out));

    let out = scrippo(&dir, &["tool.scrippo", "--help"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("run"), "{}", stdout(&out));
    assert!(!stdout(&out).contains("scrippo-core"), "{}", stdout(&out));
}

#[test]
fn completion_offers_modes_then_only_what_fits() {
    let all = complete(&[]);
    for flag in ["-n", "--new", "-ls", "--list", "-rm", "--remove", "-i", "--install", "-c", "--compile", "--hook", "--complete", "--help"] {
        assert!(all.iter().any(|c| c == flag), "{flag} in {all:?}");
    }
    // Modifiers only make sense once a mode is chosen.
    for flag in ["-p", "--package", "-g", "--tag"] {
        assert!(!all.iter().any(|c| c == flag), "{flag} in {all:?}");
    }
    assert!(!all.iter().any(|c| c == "--local"), "{all:?}");

    let new = complete(&["--new"]);
    assert!(new.contains(&"--package".into()) && new.contains(&"-g".into()), "{new:?}");
    assert!(!new.contains(&"--local".into()), "{new:?}");
    assert!(!new.contains(&"--tag".into()), "{new:?}");
    assert!(!new.contains(&"-i".into()) && !new.contains(&"--hook".into()), "{new:?}");
    assert!(!new.contains(&"--new".into()), "{new:?}");

    let package = complete(&["-n", "-p"]);
    assert!(!package.contains(&"-s".into()) && !package.contains(&"--script".into()), "{package:?}");
    assert!(package.contains(&"-g".into()), "{package:?}");

    let list = complete(&["-ls"]);
    assert!(list.contains(&"-p".into()) && list.contains(&"-s".into()), "{list:?}");
    assert!(list.contains(&"-g".into()), "{list:?}");

    let install = complete(&["-i"]);
    assert!(install.contains(&"--tag".into()), "{install:?}");
    assert!(!complete(&["-i", "--tag", "v1"]).contains(&"--tag".into()));

    let compile = complete(&["-c"]);
    assert!(!compile.contains(&"-n".into()) && !compile.contains(&"--new".into()), "{compile:?}");
    assert!(!compile.contains(&"--noexec".into()), "{compile:?}");

    // Everything after the positional belongs to the script or the mode.
    assert!(complete(&["--new", "name"]).is_empty());
}

#[test]
fn completion_continues_into_the_named_script() {
    let dir = temp_dir("script-completion");
    let script = dir.join("app.scrippo");
    std::fs::write(&script, ".build([--release])\n  -> b\n.test()\n  -> t\n").unwrap();
    let script = script.to_str().unwrap();
    assert_eq!(complete(&[script]), vec!["build", "test"]);
    assert_eq!(complete(&["--noexec", script, "build"]), vec!["--release"]);
    // A mode flag means the word is not a script to run.
    assert!(!complete(&["--new", script]).contains(&"build".into()));
    assert!(!complete(&["-n", script]).contains(&"build".into()));
}

#[test]
fn hook_registers_completion_for_scrippo_itself() {
    let dir = temp_dir("hook");
    let config = dir.join("config");
    std::fs::write(&config, "autoload=false\nautocomplete=true\n").unwrap();
    for (shell, registration) in [
        ("bash", "complete -o filenames -o default -F _scrippo_comp scrippo"),
        ("zsh", "compdef -a _scrippo_comp scrippo"),
    ] {
        let out = Command::new(CORE)
            .arg("--env")
            .env("SCRIPPO_CONFIG", &config)
            .env("SCRIPPO_SHELL", shell)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
        let text = stdout(&out);
        assert!(text.contains("_SCRIPPO_FILES[scrippo]=@cli"), "{text}");
        assert!(text.contains(registration), "{text}");
    }
}
