use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Fixture {
    temp: TempDir,
    home: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home with spaces");
        let project = temp.path().join("project");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&project).unwrap();
        Self {
            temp,
            home,
            project,
        }
    }

    fn command(&self, source: &str) -> Command {
        let script = self.project.join("main.scrippo");
        fs::write(&script, source).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_scrippo"));
        command
            .current_dir(self.temp.path())
            .env("HOME", &self.home)
            .env("SCRIPPO_HOME", self.temp.path().join("scrippo-home"))
            .env("SCRIPPO_CONFIG", self.temp.path().join("config"))
            .arg(script);
        command
    }
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn home_imports_support_unquoted_and_quoted_paths_and_nested_relative_imports() {
    let f = Fixture::new();
    fs::create_dir(f.home.join("lib")).unwrap();
    fs::write(
        f.home.join("lib/greet.scrippo"),
        "@import ./message.scrippo\ngreet()\n  message\n",
    )
    .unwrap();
    fs::write(
        f.home.join("lib/message.scrippo"),
        "message() -> from-home\n",
    )
    .unwrap();
    fs::write(
        f.home.join("my helper.bash"),
        "raw_greet() { echo raw-home; }\n",
    )
    .unwrap();

    for path in [
        "~/lib/greet.scrippo",
        "\"~/lib/greet.scrippo\"",
        "'~/lib/greet.scrippo'",
        "~//lib/greet.scrippo",
    ] {
        let output = f
            .command(&format!("@import {path}\ngreet\n"))
            .output()
            .unwrap();
        assert_eq!(stdout(&output), "from-home\n");
    }
    let output = f
        .command("@import \"~/my helper.bash\"\nraw_greet\n")
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "raw-home\n");
}

#[test]
fn home_globs_are_sorted_and_share_import_once_with_absolute_paths() {
    let f = Fixture::new();
    fs::create_dir_all(f.home.join("lib/skipped.scrippo")).unwrap();
    fs::write(f.home.join("lib/b.scrippo"), "-> second\n").unwrap();
    fs::write(f.home.join("lib/a.scrippo"), "-> first\n").unwrap();
    for pattern in [
        "~/lib/*.scrippo",
        "\"~/lib/*.scrippo\"",
        "'~/lib/*.scrippo'",
    ] {
        let source = format!(
            "@import {pattern}\n@import \"{}\"\n",
            f.home.join("lib/a.scrippo").display()
        );
        assert_eq!(
            stdout(&f.command(&source).output().unwrap()),
            "first\nsecond\n"
        );
    }
}

#[test]
fn home_imports_work_in_command_scopes() {
    let f = Fixture::new();
    fs::write(f.home.join("commands.scrippo"), ".hello() -> scoped-home\n").unwrap();
    let output = f
        .command("@import ~/commands.scrippo --as .tools\n")
        .args(["tools", "hello"])
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "scoped-home\n");
}

#[test]
fn tilde_elsewhere_in_a_path_stays_literal() {
    let f = Fixture::new();
    fs::create_dir(f.project.join("~other")).unwrap();
    fs::write(f.project.join("~other/helper.scrippo"), "-> other\n").unwrap();
    fs::write(f.project.join("helper~.scrippo"), "-> literal\n").unwrap();
    let output = f
        .command("@import ~other/helper.scrippo\n@import ./helper~.scrippo\n")
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "other\nliteral\n");
}

#[test]
fn missing_or_empty_home_reports_an_import_error() {
    let f = Fixture::new();
    for empty in [false, true] {
        let mut command = f.command("@import ~/helper.scrippo\n");
        if empty {
            command.env("HOME", "");
        } else {
            command.env_remove("HOME");
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("main.scrippo:1:"), "{error}");
        assert!(
            error.contains("@import: cannot expand '~': HOME is not set or is empty"),
            "{error}"
        );
    }
}

#[test]
fn missing_home_files_and_globs_report_the_expanded_path() {
    let f = Fixture::new();
    for (pattern, message, expanded) in [
        (
            "~/missing.scrippo",
            "file not found",
            f.home.join("missing.scrippo"),
        ),
        ("~/*.scrippo", "no matches", f.home.join("*.scrippo")),
        ("~", "file not found", f.home.clone()),
    ] {
        let output = f.command(&format!("@import {pattern}\n")).output().unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(message), "{error}");
        assert!(error.contains(&expanded.display().to_string()), "{error}");
    }
}
