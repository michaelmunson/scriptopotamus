use scriptopotamus::macros::info::{FileInfo, FileKind, Version};
use scriptopotamus::{compile_file, compile_source, parse};
use std::path::Path;

fn metadata(source: &str) -> FileInfo {
    let path = Path::new("metadata.scrippo");
    let program = parse::parse_source(path, source).unwrap();
    FileInfo::from_program(path, &program).unwrap().unwrap()
}

#[test]
fn named_metadata_is_literal_and_emits_no_runtime_code() {
    let header = r##"#!/usr/bin/env scrippo
# Metadata may follow a shebang, blank lines, and comments.

@info(
  --name hello
  --description "A greeting (with punctuation); # hello" # comment
  --version '1.2.3'
  --author 'Ada Lovelace'
  --license MIT
  --repository "https://example.com/hello.git"
  --script
  --autoload disable
) # end of declaration
"##;
    let info = metadata(header);
    assert_eq!(info.name.as_deref(), Some("hello"));
    assert_eq!(info.version, Some(Version([1, 2, 3])));
    assert_eq!(info.description, "A greeting (with punctuation); # hello");
    assert_eq!(info.author.as_deref(), Some("Ada Lovelace"));
    assert_eq!(info.license.as_deref(), Some("MIT"));
    assert_eq!(
        info.repository.as_deref(),
        Some("https://example.com/hello.git")
    );
    assert_eq!(info.kind, Some(FileKind::Script));
    assert!(!info.autoload);
    let body = ".hello() -> hello\n";
    let path = Path::new("metadata.scrippo");
    assert_eq!(
        compile_source(path, &(header.to_string() + body)).unwrap(),
        compile_source(path, body).unwrap()
    );
}

#[test]
fn metadata_options_are_optional_for_ordinary_files() {
    for header in [
        "@info()",
        "@info(\n)",
        "@info",
        "@info --description 'just a description'",
    ] {
        let info = metadata(header);
        assert!(info.name.is_none());
        assert!(info.version.is_none());
        assert!(info.autoload);
        assert!(info.kind.is_none());
    }
    assert_eq!(
        metadata("@info(--package --autoload enable)").kind,
        Some(FileKind::Package)
    );
    let old = metadata("@info hello 1.2.3 'A greeting'");
    let named = metadata("@info(--name hello --version 1.2.3 --description 'A greeting')");
    assert_eq!(old.name, named.name);
    assert_eq!(old.version, named.version);
    assert_eq!(old.description, named.description);
}

#[test]
fn invalid_metadata_reports_source_locations() {
    for (source, message) in [
        ("@info(--unknown value)", "unknown option"),
        ("@info(--name)", "requires a value"),
        ("@info(--name --script)", "requires a value"),
        ("@info(--name hello --name hello)", "duplicate option"),
        ("@info(--script --script)", "duplicate option"),
        ("@info(--script --package)", "mutually exclusive"),
        ("@info(--autoload true)", "enable or disable"),
        ("@info(--name ../hello)", "invalid name"),
        ("@info(--name 'bad name')", "invalid name"),
        ("@info(--version 01.2.3)", "stable version"),
        ("@info(--version 1.2.3-beta)", "stable version"),
        ("@info(--author \"$USER\")", "values must be literals"),
        (
            "@info(--description \"$(touch sentinel)\")",
            "values must be literals",
        ),
        ("@info(--license \"`whoami`\")", "values must be literals"),
        (
            "@info(--description \"unterminated)",
            "unterminated literal",
        ),
        ("@info(\n  --script\n", "closing ')'"),
        ("@info(--script) extra", "unexpected text"),
        ("@info()\n@info()", "exactly one top-level @info"),
        ("-> hello\n@info()", "top of the file"),
        ("@import missing.scrippo\n@info()", "top of the file"),
        ("f()\n  @info()", "top of the file"),
        ("f() -> hi\nf()\n  @info()", "top of the file"),
        ("if true; @info --script", "top of the file"),
        ("if true\n  @info()", "top of the file"),
        ("for x in a\n  @info()", "top of the file"),
        ("while true\n  @info()", "top of the file"),
        ("case x\n  x)\n    @info()", "top of the file"),
        ("@custom()\n  @info()", "top of the file"),
    ] {
        let error = compile_source(Path::new("bad-info.scrippo"), source)
            .unwrap_err()
            .to_string();
        assert!(error.contains("bad-info.scrippo:"), "{source}: {error}");
        assert!(error.contains(message), "{source}: {error}");
    }
}

#[test]
fn imported_files_own_their_metadata_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("child.scrippo");
    let main = dir.path().join("main.scrippo");
    std::fs::write(&child, "@info(--name child --package)\nhello() -> hello\n").unwrap();
    std::fs::write(
        &main,
        "@info(--name main --script)\n@import child.scrippo --as .child\n",
    )
    .unwrap();
    let bash = compile_file(&main).unwrap();
    assert!(!bash.contains("@info"));
    assert!(bash.contains("hello"));
    std::fs::write(&child, "hello() -> hello\n@info(--package)\n").unwrap();
    let error = compile_file(&main).unwrap_err().to_string();
    assert!(error.contains("child.scrippo:2"), "{error}");
}
