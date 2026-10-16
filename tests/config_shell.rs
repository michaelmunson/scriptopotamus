use scriptopotamus::config::Config;
use scriptopotamus::parse;
use scriptopotamus::shell;
use std::path::Path;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    assert_cmd_bin()
}

fn assert_cmd_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("scrippo");
    path
}

#[test]
fn config_roundtrip() {
    let dir = tempfile_dir();
    let path = dir.join("config");
    unsafe {
        std::env::set_var("SCRIPPO_CONFIG", &path);
    }
    let mut cfg = Config::default();
    cfg.autoload = true;
    cfg.autocomplete = true;
    cfg.save().unwrap();
    let loaded = Config::load().unwrap();
    assert_eq!(loaded.autoload, true);
    assert_eq!(loaded.autocomplete, true);
    unsafe {
        std::env::remove_var("SCRIPPO_CONFIG");
    }
}

#[test]
fn complete_commands_and_flags() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/commands.scrippo");
    let program = parse::parse_file(&path).unwrap();
    let top = shell::completion_candidates_for_test(&program, &[]);
    assert!(top.iter().any(|c| c == "greet"));
    assert!(top.iter().any(|c| c == "math"));
    let flags = shell::completion_candidates_for_test(&program, &["greet".into()]);
    assert!(flags.iter().any(|c| c == "--loud"));
    let subs = shell::completion_candidates_for_test(&program, &["math".into()]);
    assert!(subs.iter().any(|c| c == "add"));
    assert!(subs.iter().any(|c| c == "mul"));
}

#[test]
fn env_emits_aliases_for_dot_scrippo_and_package() {
    let dir = tempfile_dir();
    std::fs::write(
        dir.join(".scrippo"),
        ".hello()\n  -> \"hi\"\n.bye()\n  -> \"bye\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("package.scrippo"),
        "#! /usr/bin/env scrippo\n.greet()\n  -> \"yo\"\n",
    )
    .unwrap();
    let cfg = dir.join("config");
    std::fs::write(&cfg, "autoload=true\nautocomplete=false\n").unwrap();

    let output = Command::new(bin())
        .env("SCRIPPO_CONFIG", &cfg)
        .env("BASH_VERSION", "5.2")
        .env_remove("ZSH_VERSION")
        .current_dir(&dir)
        .arg("--env")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alias package="), "{stdout}");
    assert!(stdout.contains("alias hello="), "{stdout}");
    assert!(stdout.contains("alias bye="), "{stdout}");
    assert!(stdout.contains("scrippo ") && stdout.contains("package.scrippo"), "{stdout}");
}

fn tempfile_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "scrippo-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
