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
    path.push("scrippo-core");
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
fn complete_includes_imported_commands() {
    let dir = tempfile_dir();
    std::fs::write(dir.join("cmd.scrippo"), ".cmd()\n  -> \"hi\"\n").unwrap();
    std::fs::write(
        dir.join("demo.scrippo"),
        "@import cmd.scrippo\n\n.demo()\n  -> \"hello\"\n",
    )
    .unwrap();
    let demo = dir.join("demo.scrippo");
    let top = shell::completion_candidates_from_file(&demo, &[]);
    assert!(top.iter().any(|c| c == "cmd"), "{top:?}");
    assert!(top.iter().any(|c| c == "demo"), "{top:?}");
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

#[test]
fn env_inherits_parent_aliases_with_nearest_directory_winning() {
    let root = tempfile_dir();
    let child = root.join("src");
    let grandchild = child.join("nested");
    std::fs::create_dir_all(&grandchild).unwrap();
    std::fs::write(root.join("_.scrippo"), ".version()\n  -> parent\n").unwrap();
    std::fs::write(root.join("shared.scrippo"), "-> parent\n").unwrap();
    std::fs::write(child.join("shared.scrippo"), "-> child\n").unwrap();
    std::fs::write(root.join(".scrippo"), ".build()\n  -> parent\n.deploy()\n  -> parent\n").unwrap();
    std::fs::create_dir(child.join(".scrippo")).unwrap();
    std::fs::write(child.join(".scrippo/commands.scrippo"), ".build()\n  -> child\n").unwrap();
    let cfg = root.join("config");
    std::fs::write(&cfg, "autoload=true\nautocomplete=true\n").unwrap();
    for shell in ["bash", "zsh"] {
        for cwd in [&child, &grandchild] {
            let output = Command::new(bin())
                .env("SCRIPPO_CONFIG", &cfg)
                .env("SCRIPPO_HOME", root.join("global"))
                .env("SCRIPPO_SHELL", shell)
                .current_dir(cwd)
                .arg("--env")
                .output()
                .unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let stdout = String::from_utf8_lossy(&output.stdout);
            for (name, file) in [
                ("_", root.join("_.scrippo")),
                ("shared", child.join("shared.scrippo")),
                ("build", child.join(".scrippo/commands.scrippo")),
                ("deploy", root.join(".scrippo")),
            ] {
                let aliases: Vec<_> = stdout.lines().filter(|line| line.starts_with(&format!("alias {name}="))).collect();
                assert_eq!(aliases.len(), 1, "{stdout}");
                assert!(aliases[0].contains(&file.display().to_string()), "{}", aliases[0]);
                assert!(stdout.contains(&format!("_SCRIPPO_FILES[{name}]={}", file.display())), "{stdout}");
                let completion = if shell == "bash" { "complete -o default -F _scrippo_comp" } else { "compdef -a _scrippo_comp" };
                assert!(stdout.contains(&format!("{completion} {name}")), "{stdout}");
            }
        }
    }
}

#[test]
fn file_metadata_controls_autoload_for_local_global_and_project_commands() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let global = root.join("global");
    std::fs::create_dir_all(global.join("scripts")).unwrap();
    std::fs::create_dir_all(root.join(".scrippo")).unwrap();
    for path in [
        root.join("hidden.scrippo"),
        global.join("scripts/hidden_global.scrippo"),
    ] {
        std::fs::write(path, "@info(--autoload disable)\n-> hello\n").unwrap();
    }
    std::fs::write(
        root.join("visible.scrippo"),
        "@info(--autoload enable)\n-> hello\n",
    )
    .unwrap();
    std::fs::write(root.join("defaulted.scrippo"), "@info()\n-> hello\n").unwrap();
    std::fs::write(root.join("plain.scrippo"), "-> hello\n").unwrap();
    std::fs::write(
        root.join(".scrippo/commands.scrippo"),
        "@info(--autoload disable)\n.hidden_cmd() -> hello\n",
    )
    .unwrap();
    let cfg = root.join("config");
    std::fs::write(&cfg, "autoload=true\nautocomplete=true\n").unwrap();
    for shell in ["bash", "zsh"] {
        let output = Command::new(bin())
            .env("SCRIPPO_CONFIG", &cfg)
            .env("SCRIPPO_HOME", &global)
            .env("SCRIPPO_SHELL", shell)
            .current_dir(root)
            .arg("--env")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        for name in ["visible", "defaulted", "plain"] {
            assert!(stdout.contains(&format!("alias {name}=")), "{stdout}");
        }
        for name in ["hidden", "hidden_global", "hidden_cmd"] {
            assert!(!stdout.contains(&format!("alias {name}=")), "{stdout}");
        }
    }
    let output = Command::new(bin())
        .current_dir(root)
        .arg("hidden.scrippo")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
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

#[test]
fn configure_alias_roundtrip_and_validation() {
    let dir = tempfile_dir();
    let cfg = dir.join("config");
    let run = |args: &[&str]| {
        Command::new(bin())
            .env("SCRIPPO_CONFIG", &cfg)
            .arg("--configure")
            .args(args)
            .output()
            .unwrap()
    };
    let out = run(&["alias=s,sc"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("alias=s,sc"));
    assert!(std::fs::read_to_string(&cfg).unwrap().contains("alias=s,sc"));

    assert!(!run(&["alias=bad name"]).status.success());
    assert!(!run(&["alias=1x"]).status.success());

    let out = run(&["alias="]);
    assert!(out.status.success());
    assert!(std::fs::read_to_string(&cfg).unwrap().contains("alias=\n"));
}

#[test]
fn env_emits_alias_with_completion() {
    let dir = tempfile_dir();
    let cfg = dir.join("config");
    std::fs::write(&cfg, "autoload=false\nautocomplete=true\nalias=s\n").unwrap();
    for (shell, registration) in [
        ("bash", "complete -o filenames -o default -F _scrippo_comp s"),
        ("zsh", "compdef -a _scrippo_comp s"),
    ] {
        let out = Command::new(bin())
            .arg("--env")
            .env("SCRIPPO_CONFIG", &cfg)
            .env("SCRIPPO_SHELL", shell)
            .current_dir(&dir)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("alias s=scrippo"), "{text}");
        assert!(text.contains("_SCRIPPO_FILES[s]=@cli"), "{text}");
        assert!(text.contains(registration), "{text}");
    }

    // Without autocomplete the alias still exists, but has no completion.
    std::fs::write(&cfg, "autoload=false\nautocomplete=false\nalias=s\n").unwrap();
    let out = Command::new(bin())
        .arg("--env")
        .env("SCRIPPO_CONFIG", &cfg)
        .env("SCRIPPO_SHELL", "bash")
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("alias s=scrippo"), "{text}");
    assert!(!text.contains("_SCRIPPO_FILES[s]"), "{text}");
}
