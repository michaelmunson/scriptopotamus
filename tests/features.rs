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
fn parent_body_runs_when_no_subcommand_is_chosen() {
    let src = ".build([--install])\n  -> parent $install\n  .extension()\n    -> extension\n";
    assert_eq!(stdout(&run(src, &["build"])), "parent false\n");
    assert_eq!(stdout(&run(src, &["build", "--install"])), "parent true\n");
    assert_eq!(stdout(&run(src, &["build", "extension"])), "extension\n");

    let src = ".build()\n  -> parent\n  .extension()\n    -> extension\n";
    assert_eq!(stdout(&run(src, &["build"])), "parent\n");
    let out = run(src, &["build", "nope"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("unknown command"));

    let src = ".math()\n  .add()\n    -> add\n";
    let out = run(src, &["math"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("Usage:"));
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

#[test]
fn nested_exclusive_sets_are_each_checked() {
    let src = ".cmd([[-a | -b] | [-c]])\n  -> ok\n";
    let out = run(src, &["cmd", "-a", "-b"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("-a and -b are mutually exclusive"),
        "{}",
        stderr(&out)
    );
    assert!(!run(src, &["cmd", "-a", "-c"]).status.success());
    assert!(run(src, &["cmd", "-b"]).status.success());
}

#[test]
fn rest_takes_everything_after_the_last_positional_verbatim() {
    let src = ".cmd(file [-v] *rest)\n  -> \"$v|$file|${rest[*]}\"\n";
    assert_eq!(
        stdout(&run(src, &["cmd", "-v", "a", "-v", "--x", "--", "b"])),
        "true|a|-v --x -- b\n"
    );
    assert_eq!(
        stdout(&run(src, &["cmd", "--other", "a", "b"])),
        "false|a|--other b\n"
    );
}

#[test]
fn type_helpers_follow_refs() {
    let src = r#"
@import scrippo/std/types
mylist<list>=(a b c)
list --length mylist
list --join "." mylist
mydict<dict>=(a=1 b=2 c=3)
dict --length mydict
dict --keys mydict
dict --values mydict
mystr<str>="a.b.c"
str --length mystr
str --split mystr "."
str mystr
"#;
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    let lines: Vec<_> = stdout(&out).lines().map(str::to_string).collect();
    assert_eq!(lines[0], "3");
    assert_eq!(lines[1], "a.b.c");
    assert_eq!(lines[2], "3");
    let mut keys: Vec<_> = lines[3].split_whitespace().collect();
    keys.sort();
    assert_eq!(keys, ["a", "b", "c"]);
    let mut values: Vec<_> = lines[4].split_whitespace().collect();
    values.sort();
    assert_eq!(values, ["1", "2", "3"]);
    assert_eq!(lines[5], "5");
    assert_eq!(lines[6], "a b c");
    assert_eq!(lines[7], "a.b.c");
}

#[test]
fn type_helpers_reverse_and_contain() {
    let src = r#"
@import scrippo/std/types
mylist<list>=(a b c)
list --reverse mylist
list mylist --contains b
list --contains mylist z
list --contains mylist "b*"
words<list>=("a b" c)
list --contains words "a b"
list --contains words "a"
empty<list>=()
list --reverse empty
list --contains empty ""
mystr<str>="a.b.c"
str --reverse mystr
str mystr --contains "b."
str --contains mystr z
str --contains mystr "*"
spaced<str>="ab c"
str --reverse spaced
str --contains spaced ""
blank<str>=""
str --reverse blank
mydict<dict>=(a=1 b=2 "c d"=3)
dict --contains mydict a
dict mydict --contains z
dict --contains mydict 1
dict --contains mydict "c d"
blankdict<dict>=(empty=)
dict --contains blankdict empty
stars<dict>=("*"=1)
dict --contains stars "*"
"#;
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "c b a\ntrue\nfalse\nfalse\ntrue\nfalse\n\nfalse\nc.b.a\ntrue\nfalse\nfalse\nc ba\ntrue\n\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn ref_binds_empty_lists_and_the_parameter_name() {
    let src = r#"
@import scrippo/std/types
empty<list>=()
list --length empty
array<list>=(a b c)
len(array<ref list>)
  -> ${#array[@]}
len array
"#;
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "0\n3\n");
}

#[test]
fn ref_declaration_follows_its_target() {
    let src = "items<list>=(a b c)\nview<ref list>=items\n-> ${#view[@]}\nview=z\n-> ${items[0]}\n";
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "3\nz\n");
}

#[test]
fn repeatable_ref_is_rejected() {
    let err = compile_source(Path::new("bad"), "f(items<ref list>...)\n  -> x\n").unwrap_err();
    assert!(err.to_string().contains("cannot be repeatable"), "{err}");
    let err = compile_source(Path::new("bad"), "alias<ref list>=(a b)\n").unwrap_err();
    assert!(err.to_string().contains("expects a variable name"), "{err}");
}

#[test]
fn usage_nests_groups_and_bullets_alternatives() {
    let src = ".cmd(\n  [--sel<str>...\n    [--multi]\n  ] | [\n    --confirm\n  ]\n)\n  -> ok\n";
    let out = run(src, &["cmd", "--help"]);
    let text = stdout(&out);
    assert!(
        text.contains(
            "    one of:\n      - [--sel]...<str>\n          [--multi]\n      - [--confirm]\n"
        ),
        "{text}"
    );
}

#[test]
fn multiline_collections_allow_comments_quotes_and_nested_blocks() {
    let src = r###"
user<dict>=(name="Ada Lovelace"
  # Dictionary entries are whitespace-separated.
  year=1815
  punctuation="(#); literal"
)
items<list>=(
  one "two words"
  "literal # hash" # inline comment
  "(parentheses)"
)
show()
  local_items<list>=(
    a
    b
  )
  -> "$local_items[1]"
  -> "$user[name]"
  -> "$user[punctuation]"
  for item in $items
    -> "$item"
show
-> "after"
"###;
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "b\nAda Lovelace\n(#); literal\none\ntwo words\nliteral # hash\n(parentheses)\nafter\n"
    );
}

#[test]
fn multiline_collection_errors_keep_source_locations() {
    for (src, expected) in [
        ("# heading\nx<list>=(\n a\n", "unterminated collection"),
        (
            "# heading\nx<str>=\"first\nlast\n",
            "unterminated string literal",
        ),
        (
            "# heading\nx<list>=(\n a\n) bad\n",
            "unexpected text after collection",
        ),
    ] {
        let error = compile_source(Path::new("multi.scrippo"), src)
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        if expected.starts_with("unterminated") {
            assert!(error.contains("multi.scrippo:2:"), "{error}");
        }
    }
    let error = compile_source(
        Path::new("multi.scrippo"),
        "# comment\n\nx<list>=(\na\n)\nfi\n",
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("multi.scrippo:6:"), "{error}");
}

#[test]
fn multiline_collections_preserve_quoted_lines_and_escaped_quotes() {
    let src = r####"items<list>=(
  "say \"hello\" (#)"
  "first
###
last"
)
-> "$items[0]"
-> "$items[1]"
"####;
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "say \"hello\" (#)\nfirst\n###\nlast\n");
}

#[test]
fn multiline_strings_preserve_newlines_whitespace_and_shell_quoting() {
    let src = r###"name<str>=Ada
message<str>="Hello $name
  indented

# literal text; (not code)
last line"
raw<str>='literal $name
and "quotes"'
-> "$message"
-> "$raw"
-> "first output line
second output line"
-> "after"
"###;
    let out = run(src, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "Hello Ada\n  indented\n\n# literal text; (not code)\nlast line\nliteral $name\nand \"quotes\"\nfirst output line\nsecond output line\nafter\n"
    );
}

const HOOKED: &str = "@hook exec
  echo script-before
@hook exit -a code=CODE
  echo \"script-after $CODE\"

.cmd()
  @hook exec
    echo cmd-before
  @hook exit code
    echo \"cmd-after $code\"
  -> body

  .sub()
    @hook exec
      echo sub-before
    @hook exit code=status
      echo \"sub-after $status\"
    -> sub-body

  .fail()
    bash -c 'exit 3'
";

#[test]
fn exec_hooks_run_around_commands_outermost_first_and_unwind_inside_out() {
    let out = run(HOOKED, &["cmd"]);
    assert_eq!(
        stdout(&out),
        "script-before\ncmd-before\nbody\ncmd-after 0\nscript-after 0\n"
    );
    assert!(out.status.success());

    let out = run(HOOKED, &["cmd", "sub"]);
    assert_eq!(
        stdout(&out),
        "script-before\ncmd-before\nsub-before\nsub-body\nsub-after 0\ncmd-after 0\nscript-after 0\n"
    );
}

#[test]
fn exit_hooks_receive_the_exit_status_and_keep_it() {
    let out = run(HOOKED, &["cmd", "fail"]);
    assert_eq!(out.status.code(), Some(3));
    assert!(stdout(&out).contains("cmd-after 3\nscript-after 3\n"), "{}", stdout(&out));
}

#[test]
fn hooks_do_not_run_for_help() {
    let out = run(HOOKED, &["--help"]);
    assert!(out.status.success());
    assert!(!stdout(&out).contains("script-before"));

    let out = run(HOOKED, &["cmd", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.starts_with("script-before\nUsage:"), "{text}");
    assert!(!text.contains("cmd-before") && !text.contains("cmd-after"), "{text}");
    assert!(text.ends_with("script-after 0\n"), "{text}");
}

#[test]
fn hook_errors_are_reported() {
    for (source, message) in [
        ("@hook\n", "expected a hook name"),
        ("@hook nope\n  echo x\n.a() -> a\n", "unknown hook 'nope'; expected exec, exit, pre-install, or post-install"),
        (".a()\n  @hook pre-install\n    echo x\n", "only allowed at the top level of a file"),
        ("@hook post-install code\n  echo x\n", "unknown argument 'code' (available: none)"),
        ("@hook exec --finally\n  echo x\n.a() -> a\n", "unknown option '--finally'"),
        ("@hook exec\n", "expected an indented body"),
        ("@hook exec --bogus\n  echo x\n", "unknown option '--bogus'"),
        ("@hook exec code\n  echo x\n.a() -> a\n", "unknown argument 'code' (available: none)"),
        ("@hook exit nope\n  echo x\n.a() -> a\n", "unknown argument 'nope' (available: code)"),
        ("@hook exit code=A code=A\n  echo x\n.a() -> a\n", "duplicate argument 'A'"),
        ("@hook exec\n  echo x\n", "needs at least one command"),
        ("f()\n  @hook exec\n    echo x\n.a() -> a\n", "directly inside a command"),
        (".a()\n  if true\n    @hook exec\n      echo x\n", "only allowed at the top level"),
    ] {
        let error = compile_source(Path::new("hooks.scrippo"), source)
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{source:?}: {error}");
    }
}

#[test]
fn install_hooks_are_not_compiled_into_scripts() {
    let out = run(
        "@hook pre-install\n  echo pre\n@hook post-install\n  echo post\n.a() -> a\n",
        &["a"],
    );
    assert_eq!(stdout(&out), "a\n");
}
