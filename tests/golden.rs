use pretty_assertions::assert_eq;
use scriptopotamus::compile_file;
use std::path::Path;
use std::process::Command;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn assert_bash_n(script: &str) {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "scrippo-gold-{}-{}.bash",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&path, script).unwrap();
    let status = Command::new("bash").arg("-n").arg(&path).status().unwrap();
    let _ = std::fs::remove_file(&path);
    assert!(status.success(), "bash -n failed for compiled script");
}

#[test]
fn compile_simple() {
    let bash = compile_file(&fixture("simple.scrippo")).unwrap();
    assert!(bash.contains("declare -r PI=3.14"));
    assert!(bash.contains("function circumference"));
    assert!(bash.contains("bc <<<"));
    assert_bash_n(&bash);
}

#[test]
fn compile_control() {
    let bash = compile_file(&fixture("control.scrippo")).unwrap();
    assert!(bash.contains("if [[ $flag == true ]]; then"));
    assert!(bash.contains("for i in \"${items[@]}\"; do"));
    assert!(bash.contains("case hello in"));
    assert_bash_n(&bash);
}

#[test]
fn compile_commands() {
    let bash = compile_file(&fixture("commands.scrippo")).unwrap();
    assert!(bash.contains("function greet"));
    assert!(bash.contains("function math__add"));
    assert!(bash.contains("case $1 in"));
    assert!(bash.contains("greet)"));
    assert_bash_n(&bash);
}

#[test]
fn compile_macros() {
    let bash = compile_file(&fixture("macros.scrippo")).unwrap();
    assert!(bash.contains("echo \"hi\""));
    assert!(!bash.contains("@say_hi"));
    assert_bash_n(&bash);
}

#[test]
fn compile_imports() {
    let bash = compile_file(&fixture("imports.scrippo")).unwrap();
    assert!(bash.contains("function throw"));
    assert!(bash.contains("function prt"));
    assert!(bash.contains("function here"));
    assert!(bash.contains("_scrippo_prt"));
    assert!(bash.contains("_SCRIPPO_FILE"));
    assert_bash_n(&bash);
}

#[test]
fn compile_params() {
    let bash = compile_file(&fixture("params.scrippo")).unwrap();
    assert!(bash.contains("-s|--select)"));
    assert!(bash.contains("--options)"));
    assert!(bash.contains("--confirm)"));
    assert!(bash.contains("[-s --options --multi] and --confirm are mutually exclusive"));
    assert!(!bash.contains("<path> and"));
    assert!(!bash.contains("-v and"));
    assert_bash_n(&bash);
}

#[test]
fn exclusive_binds_only_neighbors() {
    let bash = scriptopotamus::compile_source(
        Path::new("input.scrippo"),
        r#"input(
  prompt
  [-v<str>]
  [
    --select<str>...
    [--multi]
  ] | [
    --confirm
  ]
)
  -> "ok"
"#,
    )
    .unwrap();
    assert!(bash.contains("[--select --multi] and --confirm are mutually exclusive"));
    assert!(!bash.contains("-v and"));
    assert!(!bash.contains("<prompt> and"));
    assert_bash_n(&bash);

    let allow = format!("{bash}\ninput hello -v name --confirm\n");
    let allow_out = Command::new("bash").arg("-c").arg(&allow).output().unwrap();
    assert!(
        allow_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&allow_out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&allow_out.stdout).trim(), "ok");

    let deny = format!("{bash}\ninput hello -v name --select a --confirm\n");
    let deny_out = Command::new("bash").arg("-c").arg(&deny).output().unwrap();
    assert!(!deny_out.status.success());
    let err = String::from_utf8_lossy(&deny_out.stderr);
    assert!(
        err.contains("[--select --multi] and --confirm are mutually exclusive"),
        "{err}"
    );
}

#[test]
fn reject_orphan_else() {
    let err =
        scriptopotamus::compile_source(Path::new("t.scrippo"), "else\n  -> \"no\"\n").unwrap_err();
    assert!(err.message.contains("without matching 'if'"));
}

#[test]
fn reject_fi() {
    let err = scriptopotamus::compile_source(Path::new("t.scrippo"), "fi\n").unwrap_err();
    assert!(err.message.contains("not valid"));
}

#[test]
fn inline_if_compiles() {
    let bash =
        scriptopotamus::compile_source(Path::new("t.scrippo"), "if $x; -> \"a\"; else -> \"b\"\n")
            .unwrap();
    assert!(bash.contains("echo \"a\""));
    assert!(bash.contains("echo \"b\""));
    assert!(bash.contains("if [[ $x == true ]]; then\n  echo \"a\"\nelse\n  echo \"b\"\nfi"));
    assert_bash_n(&bash);
}
