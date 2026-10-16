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
    assert!(bash.contains("_scrippo_prt"));
    assert_bash_n(&bash);
}

#[test]
fn compile_params() {
    let bash = compile_file(&fixture("params.scrippo")).unwrap();
    assert!(bash.contains("-s|--select)"));
    assert!(bash.contains("--options)"));
    assert!(bash.contains("--confirm)"));
    assert!(bash.contains("mutually exclusive"));
    assert_bash_n(&bash);
}

#[test]
fn reject_orphan_else() {
    let err = scriptopotamus::compile_source(
        Path::new("t.scrippo"),
        "else\n  -> \"no\"\n",
    )
    .unwrap_err();
    assert!(err.message.contains("without matching 'if'"));
}

#[test]
fn reject_fi() {
    let err = scriptopotamus::compile_source(Path::new("t.scrippo"), "fi\n").unwrap_err();
    assert!(err.message.contains("not valid"));
}

#[test]
fn inline_if_compiles() {
    let bash = scriptopotamus::compile_source(
        Path::new("t.scrippo"),
        "if $x; -> \"a\"; else -> \"b\"\n",
    )
    .unwrap();
    assert!(bash.contains("echo \"a\""));
    assert!(bash.contains("echo \"b\""));
    assert_eq!(bash.matches("fi").count(), 1);
    assert_bash_n(&bash);
}
