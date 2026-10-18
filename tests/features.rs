use scriptopotamus::{compile_file, compile_source, shell::completion_candidates_from_file};
use std::{
    io::Write,
    path::Path,
    process::{Command, Output},
};

fn run(source: &str, args: &[&str]) -> Output {
    let bash = compile_source(Path::new("features.scrippo"), source).unwrap();
    run_bash(&bash, args)
}
fn run_bash(bash: &str, args: &[&str]) -> Output {
    let mut child = Command::new("bash")
        .arg("-s")
        .arg("--")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(bash.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn aliases_are_distinct_from_exclusive_options() {
    let src = ".cmd([-ls/--list | -rm / --remove])\n  if $--list\n    -> list\n  if $-rm\n    -> remove\n";
    for flag in ["-ls", "--list"] {
        let out = run(src, &["cmd", flag]);
        assert!(out.status.success(), "{}", stderr(&out));
        assert_eq!(stdout(&out), "list\n");
    }
    assert_eq!(stdout(&run(src, &["cmd", "--remove"])), "remove\n");
    let out = run(src, &["cmd", "-ls", "-rm"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("mutually exclusive"));
    assert!(compile_source(Path::new("bad"), ".cmd([--a /])\n").is_err());
}

#[test]
fn inherit_binds_params_and_options_at_each_depth() {
    let src =
        ".path(path [--loud])\n  .rel(@inherit)\n    .deep(@inherit)\n      -> $path $--loud\n";
    let out = run(src, &["path", "rel", "deep", "hello", "--loud"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "hello true\n");
    assert!(compile_source(Path::new("bad"), ".cmd(@inherit)\n").is_err());
}

#[test]
fn usage_documents_scopes_and_inherited_params() {
    let src = "#? get a path\n.path(\n  #? input path\n  path\n  [--count<int=2>] #? number of paths\n)\n  #? relative path\n  .rel(@inherit)\n    -> $path\n";
    let out = run(src, &["--help"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("path # get a path"));
    let out = run(src, &["path", "rel", "--help"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("path rel # relative path"), "{text}");
    assert!(text.contains("{path} # input path"), "{text}");
    assert!(text.contains("[--count]<int> # number of paths"), "{text}");
    assert!(!text.contains('\x1b'));
    let out = run(src, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("Usage:"));
}

#[test]
fn usage_exit_and_explicit_help_override() {
    let out = run("f()\n  @usage 7\nf\n", &[]);
    assert_eq!(out.status.code(), Some(7));
    assert!(stdout(&out).contains("f"));
    let out = run("@usage --no-exit\n-> continued\n", &[]);
    assert!(out.status.success());
    assert!(stdout(&out).ends_with("continued\n"));
    let out = run(".cmd([--help])\n  -> custom $--help\n", &["cmd", "--help"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "custom true\n");
}

#[test]
fn scoped_imports_dispatch_and_complete_without_leaking_commands() {
    let dir = std::env::temp_dir().join(format!("scrippo-scopes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("child.scrippo"),
        ".version([--short])\n  -> version $--short\n",
    )
    .unwrap();
    let main = dir.join("main.scrippo");
    std::fs::write(&main, "@import child.scrippo --as .one\n@import child.scrippo --as .two\n@import child.scrippo --as helper\n").unwrap();
    let bash = compile_file(&main).unwrap();
    for scope in ["one", "two"] {
        let out = run_bash(&bash, &[scope, "version", "--short"]);
        assert!(out.status.success(), "{}", stderr(&out));
        assert_eq!(stdout(&out), "version true\n");
    }
    assert!(!run_bash(&bash, &["version"]).status.success());
    assert_eq!(
        completion_candidates_from_file(&main, &[]),
        vec!["one", "two"]
    );
    assert_eq!(
        completion_candidates_from_file(&main, &["one".into()]),
        vec!["version"]
    );
    assert_eq!(
        completion_candidates_from_file(&main, &["one".into(), "version".into()]),
        vec!["--short"]
    );
    let out = run_bash(&(bash + "\nhelper version --short\n"), &["one", "version"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).ends_with("version true\n"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn help_after_arguments_skips_validation_and_unknown_subcommands_fail() {
    let out = run(
        ".cmd(value<int>)\n  -> $value\n",
        &["cmd", "oops", "--help"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("<int>"));
    let out = run(
        ".parent()\n  .child()\n    -> child\n",
        &["parent", "missing"],
    );
    assert!(!out.status.success());
    assert!(stderr(&out).contains("unknown command"));
    assert!(stderr(&out).contains("Usage:"));
}

#[test]
fn root_help_includes_params_and_explicit_help_still_wins() {
    let src = "#? Root documentation\n.(value<int>)\n  -> $value\n";
    let out = run(src, &["--help"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Root documentation"));
    assert!(stdout(&out).contains("{value}<int>"));
    let out = run(".([--help])\n  -> custom $--help\n", &["--help"]);
    assert_eq!(stdout(&out), "custom true\n");
}

#[test]
fn scoped_import_promotes_an_imported_root_command() {
    let dir = std::env::temp_dir().join(format!("scrippo-root-scope-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("child.scrippo"),
        "#? Imported root\n.(value<int>)\n  -> $value\n",
    )
    .unwrap();
    let main = dir.join("main.scrippo");
    std::fs::write(&main, "@import child.scrippo --as .child\n").unwrap();
    let bash = compile_file(&main).unwrap();
    let out = run_bash(&bash, &["child", "42"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "42\n");
    let out = run_bash(&bash, &["child", "--help"]);
    assert!(stdout(&out).contains("Imported root"));
    std::fs::remove_dir_all(dir).unwrap();
}
