use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    repo: PathBuf,
    project: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let repo = root.join("helpers.git");
        let project = root.join("project");
        let home = root.join("home");
        for dir in [&repo, &project, &home] {
            fs::create_dir_all(dir).unwrap();
        }
        let fixture = Self {
            _temp: temp,
            root,
            repo,
            project,
            home,
        };
        fixture.git(&["init", "-q"]);
        fixture.version("1.0.0");
        fixture.tag("v1.0.0");
        fixture
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.repo.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn version(&self, version: &str) {
        self.write("packages/greet.scrippo", &format!("@info greet {version} \"Greeting library\"\n@import \"../support/message.scrippo\"\n.greet()\n  message\n"));
        self.write("scripts/hello.scrippo", &format!("@info hello {version}\n@import \"../support/message.scrippo\"\n@import scrippo/std/path\n.main()\n  message\n  here\n"));
        self.write(
            "support/message.scrippo",
            &format!("message() -> \"{version}\"\n"),
        );
    }

    fn git(&self, args: &[&str]) -> Output {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo)
            .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", text(&output));
        output
    }

    fn tag(&self, tag: &str) {
        self.git(&["add", "."]);
        self.git(&["commit", "-qm", tag]);
        self.git(&["tag", "-a", "-m", tag, tag]);
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_scrippo-core"));
        command
            .current_dir(&self.project)
            .env("SCRIPPO_HOME", &self.home)
            .env("SCRIPPO_CONFIG", self.root.join("config"));
        command
    }

    fn install(&self, options: &[&str], destination: Option<&Path>) -> Output {
        let mut command = self.command();
        command.arg("-i").args(options).arg(&self.repo);
        if let Some(path) = destination {
            command.arg(path);
        }
        command.output().unwrap()
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn lib(&self) -> PathBuf {
        self.project.join(".scrippo/lib")
    }

    fn manifest(&self, directory: &Path, kind: &str) -> serde_json::Value {
        serde_json::from_slice(
            &fs::read(directory.join(format!(
                ".scrippo-installed/{kind}/helpers/content/install.json"
            )))
            .unwrap(),
        )
        .unwrap()
    }
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
fn success(output: &Output) {
    assert!(output.status.success(), "{}", text(output));
}
fn failure(output: &Output, message: &str) {
    assert!(
        !output.status.success(),
        "unexpected success: {}",
        text(output)
    );
    assert!(
        text(output).contains(message),
        "expected {message}: {}",
        text(output)
    );
}

#[test]
fn latest_local_package_is_importable_and_does_not_dispatch_the_callers_arguments() {
    let f = Fixture::new();
    f.version("1.9.0");
    f.tag("v1.9.0");
    f.version("1.10.0");
    f.tag("1.10.0");
    f.git(&["tag", "9.0.0-beta"]);
    f.version("99.0.0");
    f.git(&["add", "."]);
    f.git(&["commit", "-qm", "untagged"]);
    success(&f.install(&["-p"], None));
    assert!(f.lib().join("greet.bash").is_file());
    let manifest = f.manifest(&f.lib(), "packages");
    assert_eq!(manifest["tag"], "1.10.0");
    assert_eq!(
        manifest["commit"],
        String::from_utf8(f.git(&["rev-parse", "1.10.0^{commit}"]).stdout)
            .unwrap()
            .trim()
    );
    fs::write(
        f.project.join("app.scrippo"),
        "@import greet\n#? caller command\n.run()\n  greet\n",
    )
    .unwrap();
    let output = f.run(&["app.scrippo", "run"]);
    success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1.10.0\n");
    let help = f.run(&["app.scrippo", "--help"]);
    success(&help);
    assert!(text(&help).contains("caller command"));
}

#[test]
fn explicit_tag_and_destination_with_spaces_work() {
    let f = Fixture::new();
    f.version("2.0.0");
    f.tag("2.0.0");
    let dir = f.root.join("custom lib");
    success(&f.install(&["--package", "--tag", "v1.0.0"], Some(&dir)));
    assert_eq!(f.manifest(&dir, "packages")["tag"], "v1.0.0");
    let output = Command::new("bash")
        .arg("-c")
        .arg("source \"$1\"; greet")
        .arg("_")
        .arg(dir.join("greet.bash"))
        .output()
        .unwrap();
    success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1.0.0\n");
}

#[test]
fn global_packages_follow_scrippo_home() {
    let f = Fixture::new();
    success(&f.install(&["-p", "-g"], None));
    assert!(f.home.join("lib/greet.bash").is_file());
    fs::write(f.project.join("app.scrippo"), "@import greet\ngreet\n").unwrap();
    let output = f.run(&["app.scrippo"]);
    success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1.0.0\n");
    fs::create_dir_all(f.lib()).unwrap();
    fs::write(f.lib().join("greet.scrippo"), "greet() -> local\n").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&f.run(&["app.scrippo"]).stdout),
        "local\n"
    );
}

#[test]
fn global_scripts_preserve_sources_relative_imports_here_and_completions() {
    let f = Fixture::new();
    success(&f.install(&["--script", "--global"], None));
    let script = f.home.join("scripts/hello.scrippo");
    let canonical = script.canonicalize().unwrap();
    assert!(
        fs::read_to_string(&canonical)
            .unwrap()
            .contains("@info hello")
    );
    let output = f.run(&["hello", "main"]);
    success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("1.0.0\n{}\n", canonical.parent().unwrap().display())
    );
    let complete = f.run(&["--complete", script.to_str().unwrap(), "", ""]);
    success(&complete);
    assert!(text(&complete).contains("main"));
    fs::write(f.root.join("config"), "autoload=true\nautocomplete=true\n").unwrap();
    let aliases = f.run(&["--env"]);
    success(&aliases);
    assert!(text(&aliases).contains("alias hello="));
}

#[test]
fn local_scripts_default_to_current_directory_and_accept_explicit_directory() {
    let f = Fixture::new();
    success(&f.install(&["-s", "--local"], None));
    assert!(f.project.join("hello.scrippo").is_file());
    success(&f.run(&["hello.scrippo", "main"]));
    let dir = f.root.join("scripts elsewhere");
    success(&f.install(&["-s"], Some(&dir)));
    success(&f.run(&[dir.join("hello.scrippo").to_str().unwrap(), "main"]));
    let moved = f.root.join("moved-project");
    fs::rename(&f.project, &moved).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_scrippo-core"))
        .current_dir(&moved)
        .env("SCRIPPO_HOME", &f.home)
        .args(["hello.scrippo", "main"])
        .output()
        .unwrap();
    success(&output);
    assert!(text(&output).contains("moved-project"));
}

#[test]
fn package_compilation_never_runs_raw_bash_or_command_substitutions() {
    let f = Fixture::new();
    let sentinel = f.root.join("executed");
    f.write("packages/greet.scrippo", &format!("@info greet 2.0.0\n@import \"../support/raw.bash\"\nx=$(touch '{}')\ngreet() -> hello\n", sentinel.display()));
    f.write(
        "support/raw.bash",
        &format!("touch '{}'\n", sentinel.display()),
    );
    f.tag("2.0.0");
    success(&f.install(&["-p"], None));
    assert!(!sentinel.exists());
}

#[test]
fn repository_dependencies_and_standard_library_are_compiled_inside_the_sandbox() {
    let f = Fixture::new();
    f.write(
        "packages/greet.scrippo",
        "@info greet 2.0.0\n@import nested\n@import scrippo/std/types\ngreet()\n  nested\n",
    );
    f.write(
        "packages/nested.scrippo",
        "@info nested 2.0.0\nnested() -> dependency\n",
    );
    f.tag("2.0.0");
    success(&f.install(&["-p"], None));
    let output = Command::new("bash")
        .args(["-c", "source \"$1\"; greet", "_"])
        .arg(f.lib().join("greet.bash"))
        .output()
        .unwrap();
    success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "dependency\n");
}

#[test]
fn metadata_validation_rejects_invalid_entries_without_publishing_anything() {
    for (source, message) in [
        ("greet() -> hi\n", "exactly one top-level @info"),
        (
            "@info greet 2.0.0\n@info greet 2.0.0\n",
            "exactly one top-level @info",
        ),
        ("@info other 2.0.0\n", "name must match"),
        ("@info greet 1.0.0\n", "version must match"),
        ("@info greet 02.0.0\n", "stable version"),
        (
            "@info greet \"$(touch sentinel)\"\n",
            "values must be literals",
        ),
        ("@info greet 2.0.0\n@unknown\n", "unknown macro"),
        (
            "@info greet 2.0.0\n.() -> script\n",
            "packages must expose named",
        ),
        ("@info greet 2.0.0\necho \"unterminated\n", "bash failed"),
    ] {
        let f = Fixture::new();
        f.write("packages/greet.scrippo", source);
        f.tag("2.0.0");
        failure(&f.install(&["-p"], None), message);
        assert!(!f.lib().join("greet.bash").exists());
        assert!(!f.lib().join(".scrippo-installed").exists());
        assert_eq!(fs::read_dir(f.lib()).unwrap().count(), 0);
    }
}

#[test]
fn named_metadata_is_preserved_for_packages_and_scripts() {
    let f = Fixture::new();
    for (directory, name, kind, body) in [
        ("packages", "greet", "package", "greet() -> hello"),
        ("scripts", "hello", "script", "-> hello"),
    ] {
        f.write(&format!("{directory}/{name}.scrippo"), &format!(
            "@info(\n  --name {name}\n  --version 2.0.0\n  --description 'A greeting'\n  --author 'Ada Lovelace'\n  --license MIT\n  --repository https://example.com/helpers.git\n  --{kind}\n  --autoload disable\n)\n{body}\n"
        ));
    }
    f.tag("v2.0.0");
    for (flag, destination, directory, name, kind) in [
        ("-p", f.lib(), "packages", "greet", "package"),
        ("-s", f.project.clone(), "scripts", "hello", "script"),
    ] {
        let output = f.install(&[flag], None);
        assert!(output.status.success(), "{}", text(&output));
        let manifest = f.manifest(&destination, directory);
        let entry = &manifest["entries"][0];
        assert_eq!(entry["name"], name);
        assert_eq!(entry["version"], "2.0.0");
        assert_eq!(entry["description"], "A greeting");
        assert_eq!(entry["author"], "Ada Lovelace");
        assert_eq!(entry["license"], "MIT");
        assert_eq!(entry["repository"], "https://example.com/helpers.git");
        assert_eq!(entry["kind"], kind);
        assert_eq!(entry["autoload"], false);
    }
    let output = f.run(&["hello.scrippo"]);
    assert!(output.status.success(), "{}", text(&output));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}

#[test]
fn installable_named_metadata_requires_name_version_and_matching_kind() {
    for (header, message) in [
        ("@info(--version 2.0.0)", "require --name"),
        ("@info(--name greet)", "require --version"),
        (
            "@info(--name greet --version 2.0.0 --script)",
            "kind must match",
        ),
        (
            "-> hi\n@info(--name greet --version 2.0.0)",
            "top of the file",
        ),
    ] {
        let f = Fixture::new();
        f.write(
            "packages/greet.scrippo",
            &format!("{header}\ngreet() -> hi\n"),
        );
        f.tag("v2.0.0");
        let output = f.install(&["-p"], None);
        assert!(!output.status.success(), "{header}");
        assert!(text(&output).contains(message), "{}", text(&output));
        assert!(!f.lib().join("greet.bash").exists());
    }
}

#[test]
fn script_metadata_is_validated_and_duplicate_names_are_rejected() {
    let f = Fixture::new();
    f.write("scripts/hello.scrippo", "-> no-metadata\n");
    f.tag("2.0.0");
    failure(
        &f.install(&["-s", "-g"], None),
        "exactly one top-level @info",
    );
    f.write(
        "packages/nested/greet.scrippo",
        "@info greet 1.0.0\ngreet() -> duplicate\n",
    );
    f.tag("3.0.0");
    // Match the tag on both entries so the duplicate name is the only error.
    f.write(
        "packages/greet.scrippo",
        "@info greet 4.0.0\ngreet() -> one\n",
    );
    f.write(
        "packages/nested/greet.scrippo",
        "@info greet 4.0.0\ngreet() -> two\n",
    );
    f.tag("4.0.0");
    failure(&f.install(&["-p"], None), "duplicate packages entry name");
}

#[test]
fn a_bad_second_package_rejects_the_entire_installation() {
    let f = Fixture::new();
    f.version("2.0.0");
    f.write("packages/zbad.scrippo", "@info zbad 2.0.0\n@unknown\n");
    f.tag("2.0.0");
    failure(&f.install(&["-p"], None), "unknown macro");
    assert!(!f.lib().join("greet.bash").exists());
    assert!(!f.lib().join("zbad.bash").exists());
}

#[test]
fn outside_imports_and_host_package_dependencies_are_blocked() {
    for glob in [false, true] {
        let f = Fixture::new();
        let outside = f.root.join("outside.scrippo");
        fs::write(&outside, "secret() -> private\n").unwrap();
        let pattern = if glob {
            format!("{}/*.scrippo", f.root.display())
        } else {
            outside.display().to_string()
        };
        f.write(
            "packages/greet.scrippo",
            &format!("@info greet 2.0.0\n@import \"{pattern}\"\n"),
        );
        f.tag("2.0.0");
        failure(&f.install(&["-p"], None), "outside the package sandbox");
    }
    let f = Fixture::new();
    fs::create_dir_all(f.home.join("lib")).unwrap();
    fs::write(f.home.join("lib/private.scrippo"), "secret() -> private\n").unwrap();
    f.write(
        "packages/greet.scrippo",
        "@info greet 2.0.0\n@import private\n",
    );
    f.tag("2.0.0");
    failure(&f.install(&["-p"], None), "package not found");
}

#[test]
fn home_imports_cannot_escape_the_package_sandbox() {
    for pattern in ["~/private.scrippo", "~/*.scrippo"] {
        let f = Fixture::new();
        fs::write(f.home.join("private.scrippo"), "secret() -> private\n").unwrap();
        f.write(
            "packages/greet.scrippo",
            &format!("@info greet 2.0.0\n@import {pattern}\n"),
        );
        f.tag("2.0.0");
        let output = f
            .command()
            .env("HOME", &f.home)
            .args(["-i", "-p"])
            .arg(&f.repo)
            .output()
            .unwrap();
        failure(&output, "outside the package sandbox");
        assert!(!f.lib().join("greet.bash").exists());
    }
}

#[cfg(unix)]
#[test]
fn repository_symlinks_are_rejected_before_snapshotting() {
    let f = Fixture::new();
    std::os::unix::fs::symlink("/etc/passwd", f.repo.join("link")).unwrap();
    f.tag("2.0.0");
    failure(&f.install(&["-p"], None), "links and submodules");
}

#[test]
fn submodules_are_rejected_without_checking_them_out() {
    let f = Fixture::new();
    let commit = String::from_utf8(f.git(&["rev-parse", "HEAD"]).stdout).unwrap();
    let entry = format!("160000,{},dependency", commit.trim());
    f.git(&["update-index", "--add", "--cacheinfo", &entry]);
    f.git(&["commit", "-qm", "submodule"]);
    f.git(&["tag", "2.0.0"]);
    failure(&f.install(&["-p"], None), "links and submodules");
}

#[test]
fn simultaneous_installs_do_not_merge_or_overwrite_each_other() {
    let f = Fixture::new();
    let start = || {
        f.command()
            .args(["-i", "-p"])
            .arg(&f.repo)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    };
    let first = start();
    let second = start();
    let outputs = [
        first.wait_with_output().unwrap(),
        second.wait_with_output().unwrap(),
    ];
    assert_eq!(
        outputs
            .iter()
            .filter(|output| output.status.success())
            .count(),
        1
    );
    assert!(f.lib().join("greet.bash").is_file());
    assert_eq!(f.manifest(&f.lib(), "packages")["tag"], "v1.0.0");
    assert!(!f.lib().join(".scrippo-install.lock").exists());
    assert!(!fs::read_dir(f.lib()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".scrippo-install-")
    }));
}

#[test]
fn existing_source_or_compiled_installations_are_preserved() {
    let f = Fixture::new();
    fs::create_dir_all(f.lib()).unwrap();
    let source = f.lib().join("greet.scrippo");
    fs::write(&source, "original").unwrap();
    failure(&f.install(&["-p"], None), "destination already exists");
    assert_eq!(fs::read_to_string(&source).unwrap(), "original");
    fs::remove_file(source).unwrap();
    success(&f.install(&["-p"], None));
    let before = fs::read(f.lib().join("greet.bash")).unwrap();
    failure(&f.install(&["-p"], None), "destination already exists");
    assert_eq!(fs::read(f.lib().join("greet.bash")).unwrap(), before);
}

#[test]
fn cli_errors_are_reported_without_changing_normal_script_arguments() {
    let f = Fixture::new();
    failure(
        &f.run(&["-i", f.repo.to_str().unwrap()]),
        "requires --package or --script",
    );
    failure(
        &f.run(&["-i", "-p", "-s", f.repo.to_str().unwrap()]),
        "cannot be used",
    );
    failure(
        &f.install(&["-p", "-g"], Some(&f.root.join("other"))),
        "--global cannot",
    );
    failure(
        &f.install(&["-p", "--tag", "../bad"], None),
        "stable version",
    );
    failure(&f.install(&["-p", "--tag", "8.0.0"], None), "git failed");
    failure(&f.run(&["-i", "-p", "--new", "x"]), "cannot be used");
    fs::write(
        f.project.join("args.scrippo"),
        ".([*args])\n  -> \"${args[@]}\"\n",
    )
    .unwrap();
    let output = f.run(&["args.scrippo", "-i", "-p", "--tag", "value"]);
    success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "-i -p --tag value\n"
    );
    fs::write(f.project.join("raw.scrippo"), "printf '<%s>\\n' \"$@\"\n").unwrap();
    let output = f.run(&["raw.scrippo", "--", "-i"]);
    success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "<-i>\n");
}

#[test]
fn missing_tags_empty_selection_and_failed_clone_do_not_publish() {
    let f = Fixture::new();
    f.git(&["tag", "-d", "v1.0.0"]);
    failure(&f.install(&["-p"], None), "no stable version tags");
    f.git(&["rm", "-r", "packages"]);
    f.tag("2.0.0");
    failure(&f.install(&["-p"], None), "no .scrippo files in packages/");
    failure(
        &f.run(&["-i", "-p", f.root.join("missing.git").to_str().unwrap()]),
        "git failed",
    );
    assert!(!f.lib().join(".scrippo-installed").exists());
}

#[test]
fn plain_directory_installs_without_git() {
    let f = Fixture::new();
    let dir = plain_helpers(&f.root);
    fs::write(
        dir.join("packages/greet.scrippo"),
        "@info greet 1.2.3 \"Greeting library\"\n@import \"../support/message.scrippo\"\n.greet()\n  message\n",
    )
    .unwrap();
    fs::write(dir.join("support/message.scrippo"), "message() -> \"from-dir\"\n").unwrap();

    let output = f.command().args(["-i", "-p"]).arg(&dir).output().unwrap();
    success(&output);
    assert!(text(&output).contains("installed 1.2.3 (local)"), "{}", text(&output));
    assert!(f.lib().join("greet.bash").is_file());
    fs::write(
        f.project.join("app.scrippo"),
        "@import greet\n.()\n  greet\n",
    )
    .unwrap();
    let output = f.run(&["app.scrippo"]);
    success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "from-dir\n");
}

fn plain_helpers(root: &Path) -> PathBuf {
    let dir = root.join("helpers");
    fs::create_dir_all(dir.join("packages")).unwrap();
    fs::create_dir_all(dir.join("support")).unwrap();
    fs::write(
        dir.join("packages/greet.scrippo"),
        "@info greet 1.0.0 \"Greeting library\"\n@import \"../support/message.scrippo\"\n.greet()\n  message\n",
    )
    .unwrap();
    fs::write(dir.join("support/message.scrippo"), "message() -> \"1.0.0\"\n").unwrap();
    dir
}

#[test]
fn install_hook_runs_shared_before_and_after() {
    let f = Fixture::new();
    let dir = plain_helpers(&f.root);
    fs::write(
        dir.join(".install.scrippo"),
        "-> \"shared\"\n.before()\n  -> \"before\"\n.after()\n  -> \"after\"\n",
    )
    .unwrap();

    let output = f.command().args(["-i", "-p"]).arg(&dir).output().unwrap();
    success(&output);
    let text = text(&output);
    assert_eq!(text.match_indices("shared\n").count(), 2, "{text}");
    assert!(
        text.find("shared\n")
            .zip(text.find("before\n"))
            .zip(text.rfind("shared\n"))
            .zip(text.find("after\n"))
            .is_some_and(|(((shared1, before), shared2), after)| {
                shared1 < before && before < shared2 && shared2 < after
            }),
        "{text}"
    );
    assert!(f.lib().join("greet.bash").is_file());
}

#[test]
fn failing_before_hook_does_not_publish() {
    let f = Fixture::new();
    let dir = plain_helpers(&f.root);
    fs::write(
        dir.join(".install.scrippo"),
        ".before()\n  -> \"nope\"\n  exit 1\n.after()\n  -> \"after\"\n",
    )
    .unwrap();

    failure(
        &f.command().args(["-i", "-p"]).arg(&dir).output().unwrap(),
        ".install.scrippo before failed",
    );
    assert!(!f.lib().join(".scrippo-installed").exists());
    assert!(!f.lib().join("greet.bash").exists());
}
