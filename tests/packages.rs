use scriptopotamus::{compile_file, paths};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("scrippo-core");
    path
}

fn tempfile_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "scrippo-pkg-{}-{}",
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
fn import_local_package_from_project_lib() {
    let dir = tempfile_dir();
    let lib = dir.join(".scrippo").join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("greet.scrippo"), ".hi()\n  -> \"hello\"\n").unwrap();
    let script = dir.join("app.scrippo");
    std::fs::write(&script, "@import greet\nhi\n").unwrap();

    let bash = compile_file(&script).unwrap();
    assert!(bash.contains("hello"), "{bash}");
}

#[test]
fn import_global_package() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = tempfile_dir();
    let project = tempfile_dir();
    let lib = home.join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("util.scrippo"), ".ping()\n  -> \"pong\"\n").unwrap();
    let script = project.join("app.scrippo");
    std::fs::write(&script, "@import util\nping\n").unwrap();

    unsafe {
        std::env::set_var("SCRIPPO_HOME", &home);
    }
    let bash = compile_file(&script).unwrap();
    unsafe {
        std::env::remove_var("SCRIPPO_HOME");
    }
    assert!(bash.contains("pong"), "{bash}");
}

#[test]
fn local_package_shadows_global() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = tempfile_dir();
    let project = tempfile_dir();
    std::fs::create_dir_all(home.join("lib")).unwrap();
    std::fs::create_dir_all(project.join(".scrippo").join("lib")).unwrap();
    std::fs::write(
        home.join("lib").join("msg.scrippo"),
        ".say()\n  -> \"global\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join(".scrippo").join("lib").join("msg.scrippo"),
        ".say()\n  -> \"local\"\n",
    )
    .unwrap();
    let script = project.join("app.scrippo");
    std::fs::write(&script, "@import msg\nsay\n").unwrap();

    unsafe {
        std::env::set_var("SCRIPPO_HOME", &home);
    }
    let bash = compile_file(&script).unwrap();
    unsafe {
        std::env::remove_var("SCRIPPO_HOME");
    }
    assert!(bash.contains("local"), "{bash}");
    assert!(!bash.contains("global"), "{bash}");
}

#[test]
fn resolve_global_script_with_and_without_extension() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = tempfile_dir();
    let scripts = home.join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::write(
        scripts.join("hello.scrippo"),
        "#! /usr/bin/env scrippo\n-> \"hi\"\n",
    )
    .unwrap();

    unsafe {
        std::env::set_var("SCRIPPO_HOME", &home);
    }
    let a = paths::resolve_script_arg(std::path::Path::new("hello")).unwrap();
    let b = paths::resolve_script_arg(std::path::Path::new("hello.scrippo")).unwrap();
    unsafe {
        std::env::remove_var("SCRIPPO_HOME");
    }
    assert!(a.ends_with("hello.scrippo"));
    assert_eq!(a, b);
}

#[test]
fn new_package_local_and_script_global() {
    let project = tempfile_dir();
    let home = tempfile_dir();

    let output = Command::new(bin())
        .env("SCRIPPO_HOME", &home)
        .current_dir(&project)
        .args(["--new", "--package", "--local", "mylib"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let path = project.join(".scrippo").join("lib").join("mylib.scrippo");
    assert!(path.is_file(), "{}", String::from_utf8_lossy(&output.stdout));

    let output = Command::new(bin())
        .env("SCRIPPO_HOME", &home)
        .current_dir(&project)
        .args(["--new", "--script", "hello"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(home.join("scripts").join("hello.scrippo").is_file());
}

#[test]
fn run_global_script_by_name() {
    let home = tempfile_dir();
    let scripts = home.join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::write(
        scripts.join("hello.scrippo"),
        "#! /usr/bin/env scrippo\n-> \"from-global\"\n",
    )
    .unwrap();

    let output = Command::new(bin())
        .env("SCRIPPO_HOME", &home)
        .arg("hello")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("from-global"), "{stdout}");
}

#[test]
fn env_aliases_global_scripts() {
    let home = tempfile_dir();
    let project = tempfile_dir();
    let scripts = home.join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::write(
        scripts.join("hello.scrippo"),
        "#! /usr/bin/env scrippo\n.greet()\n  -> \"hi\"\n",
    )
    .unwrap();
    let cfg = project.join("config");
    std::fs::write(&cfg, "autoload=true\nautocomplete=false\n").unwrap();

    let output = Command::new(bin())
        .env("SCRIPPO_CONFIG", &cfg)
        .env("SCRIPPO_HOME", &home)
        .env("BASH_VERSION", "5.2")
        .env_remove("ZSH_VERSION")
        .current_dir(&project)
        .arg("--env")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alias hello="), "{stdout}");
}

#[test]
fn env_aliases_commands_from_scrippo_dir() {
    let dir = tempfile_dir();
    std::fs::create_dir_all(dir.join(".scrippo").join("lib")).unwrap();
    std::fs::write(
        dir.join(".scrippo").join("commands.scrippo"),
        ".hello()\n  -> \"hi\"\n",
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
    assert!(stdout.contains("alias hello="), "{stdout}");
    assert!(stdout.contains("commands.scrippo"), "{stdout}");
}

fn run_script(home: &PathBuf, cwd: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .env("SCRIPPO_HOME", home)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn run_local_script_by_name_from_subdirectory_overriding_global() {
    let home = tempfile_dir();
    let project = tempfile_dir();
    std::fs::create_dir_all(home.join("scripts")).unwrap();
    std::fs::write(home.join("scripts/build.scrippo"), "-> \"from-global\"\n").unwrap();
    std::fs::write(project.join("build.scrippo"), "-> \"from-local\"\n").unwrap();
    // A directory named like the script must not shadow it.
    std::fs::create_dir_all(project.join("build")).unwrap();
    let sub = project.join("build");

    for cwd in [&project, &sub] {
        let output = run_script(&home, cwd, &["build"]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "from-local");
    }

    // Nearest script wins over one further up.
    std::fs::write(sub.join("build.scrippo"), "-> \"from-sub\"\n").unwrap();
    let output = run_script(&home, &sub, &["build.scrippo"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "from-sub");

    // Without a local script, the global one is used.
    let other = tempfile_dir();
    let output = run_script(&home, &other, &["build"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "from-global");
}

#[test]
fn complete_local_script_commands_from_subdirectory() {
    let project = tempfile_dir();
    std::fs::write(project.join("build.scrippo"), ".compile()\n  -> \"c\"\n").unwrap();
    let sub = project.join("deep");
    std::fs::create_dir_all(&sub).unwrap();

    let _guard = ENV_LOCK.lock().unwrap();
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&sub).unwrap();
    let got = scriptopotamus::shell::completion_candidates_from_file(
        std::path::Path::new(scriptopotamus::shell::CLI_SELF),
        &["build".to_string()],
    );
    std::env::set_current_dir(old).unwrap();
    assert!(got.contains(&"compile".to_string()), "{got:?}");
}
