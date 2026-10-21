#!/usr/bin/env bash

source "$(dirname "$0")/test_utils.bash"
root=$(cd "$SCRIPT_DIR/../.." && pwd)
scrippo=$root/scrippo
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
failures=0
mkdir -p "$scratch/home" "$scratch/local dir"
export HOME=$scratch/home

function check {
  if expect_equal "$2" "$3"; then
    echo "ok: $1"
  else
    echo "FAIL: $1"
    failures=$(( failures + 1 ))
  fi
}

function commit_version {
  git -C "$repo" add .
  git -C "$repo" -c user.name=Test -c user.email=test@example.com commit -qm "$1"
  git -C "$repo" tag "$1"
}

function write_version {
  printf '@info is-even %s "Even number library"\n@import "../support.bash"\nis_even(value<int>)\n\t[[ $((value %% 2)) == 0 ]]\n' "$1" > "$repo/packages/is-even.scrippo"
  printf '@info greet %s\n@import "../support.bash"\n.()\n\t-> "%s"\n' "$1" "$1" > "$repo/scripts/greet.scrippo"
}

function install_to {
  "$scrippo" -i "$@" > "$scratch/install.log" 2>&1
}

function reject {
  local name=$1 expected=$2
  shift 2
  if install_to "$@"; then
    check "$name" failure success
  else
    check "$name" yes "$(grep -Fq -- "$expected" "$scratch/install.log" && echo yes)"
    [[ $(tail -n 1 "$scratch/install.log") == *"$expected"* ]] || cat "$scratch/install.log" >&2
  fi
}

repo=$scratch/example.git
git init -q "$repo"
mkdir -p "$repo/packages" "$repo/scripts"
# Imported raw Bash and top-level commands must be emitted without execution.
printf 'touch %q\n' "$scratch/executed" > "$repo/support.bash"
write_version 1.9.0
commit_version v1.9.0
write_version 1.10.0
commit_version 1.10.0
git -C "$repo" tag 9.0.0-beta
git -C "$repo" tag release
write_version 99.0.0
git -C "$repo" add .
git -C "$repo" -c user.name=Test -c user.email=test@example.com commit -qm untagged

check "install latest package" 0 "$(install_to -p "$repo" "$scratch/local dir"; echo $?)"
installed=$scratch/local\ dir/packages/example/content
check "highest version ignores prefixes, prereleases, and HEAD" 1.10.0 "$(jq -r .tag "$installed/.scrippo-install.json")"
check "installation does not execute repository code" no "$([[ -e $scratch/executed ]] && echo yes || echo no)"
check "package is fully compiled and can be sourced" yes "$(bash -c 'source "$1"; is_even 4 && ! is_even 3 && echo yes' _ "$installed/is-even.bash")"
# Running the compiled file above executes the top-level touch. Remove it and
# verify that a fresh installation itself never runs repository code.
rm -f "$scratch/executed"
check "explicit tag and long options" 0 "$(install_to --package --tag v1.9.0 "$repo" "$scratch/pinned"; echo $?)"
check "install does not execute package or import code" no "$([[ -e $scratch/executed ]] && echo yes || echo no)"
check "pinned package manifest" v1.9.0 "$(jq -r .tag "$scratch/pinned/packages/example/content/.scrippo-install.json")"
check "scripts install globally" 0 "$(install_to --script --global "$repo"; echo $?)"
script=$HOME/.scriptopotamus/scripts/example/content/scripts/greet.scrippo
check "script source is retained" yes "$([[ -f $script ]] && echo yes)"
check "script keeps relative imports" 1.10.0 "$("$scrippo" "$script")"
check "local default is current directory" 0 "$(cd "$scratch" && install_to -s --tag v1.9.0 "$repo"; echo $?)"
check "default destination exists" yes "$([[ -f $scratch/scripts/example/content/scripts/greet.scrippo ]] && echo yes)"
check "manifest records exact commit" "$(git -C "$repo" rev-parse 1.10.0)" "$(jq -r .commit "$installed/.scrippo-install.json")"
printf '.([*args])\n\t-> "${args[@]}"\n' > "$scratch/forward.scrippo"
check "script flags do not trigger installation" '-i -p --tag value' "$("$scrippo" "$scratch/forward.scrippo" -i -p --tag value)"

reject "existing installation is preserved" 'destination already exists' -p --tag v1.9.0 "$repo" "$scratch/local dir"
check "existing manifest remains intact" 1.10.0 "$(jq -r .tag "$installed/.scrippo-install.json")"
reject "mode is required" 'exactly one' "$repo"
reject "modes are exclusive" 'exactly one' -p -s "$repo"
reject "global rejects destination" '--global cannot' -p -g "$repo" "$scratch/other"
reject "unknown options are rejected" 'unknown option' -p --oops "$repo"
reject "tag requires a value" '--tag requires' -p --tag
reject "invalid tag is rejected" 'invalid version tag' -p --tag ../bad "$repo"
reject "missing tag is rejected" 'tag not found' -p --tag 8.0.0 "$repo" "$scratch/missing"
reject "failed clone reports failure" 'does not exist' -p "$scratch/missing.git" "$scratch/clone"

write_version 2.0.0
printf '%s\n' '-> missing' > "$repo/packages/is-even.scrippo"
commit_version 2.0.0
reject "missing metadata fails" 'exactly one top-level @info' -p "$repo" "$scratch/bad"
check "failed install leaves no destination" no "$([[ -e $scratch/bad/packages/example ]] && echo yes || echo no)"
printf '@info is-even 2.0.0\n@info is-even 2.0.0\n' > "$repo/packages/is-even.scrippo"
commit_version 2.0.1
reject "duplicate metadata fails" 'exactly one top-level @info' -p "$repo" "$scratch/bad"
printf '@info wrong-name 2.0.2\n' > "$repo/packages/is-even.scrippo"
commit_version 2.0.2
reject "metadata name matches filename" 'name must match' -p "$repo" "$scratch/bad"
printf '@info is-even 1.0.0\n' > "$repo/packages/is-even.scrippo"
commit_version 2.0.3
reject "metadata version matches tag" 'version must match' -p "$repo" "$scratch/bad"
printf '@info is-even "$(touch sentinel)"\n' > "$repo/packages/is-even.scrippo"
commit_version 2.0.4
reject "metadata is never evaluated" 'values must be literals' -p "$repo" "$scratch/bad"
write_version 2.0.5
printf '@import "/etc/passwd"\n' >> "$repo/packages/is-even.scrippo"
commit_version 2.0.5
reject "absolute imports cannot escape sandbox" 'outside the package sandbox' -p "$repo" "$scratch/bad"
write_version 2.0.6
printf '@import "../../../../etc/passwd"\n' >> "$repo/packages/is-even.scrippo"
commit_version 2.0.6
reject "parent traversal cannot escape sandbox" 'outside the package sandbox' -p "$repo" "$scratch/bad"
write_version 2.0.7
printf '@import "/etc/pass*"\n' >> "$repo/packages/is-even.scrippo"
commit_version 2.0.7
reject "glob imports cannot escape sandbox" 'outside the package sandbox' -p "$repo" "$scratch/bad"
write_version 2.0.8
printf 'echo "unterminated\n' >> "$repo/packages/is-even.scrippo"
commit_version 2.0.8
reject "invalid compiled Bash fails" 'unexpected EOF' -p "$repo" "$scratch/bad"
write_version 2.0.9
ln -s /etc/passwd "$repo/link"
commit_version 2.0.9
reject "symlinks are rejected before extraction" 'links and submodules' -p "$repo" "$scratch/bad"
rm "$repo/link"
write_version 2.0.10
rm "$repo/scripts/greet.scrippo"
commit_version 2.0.10
reject "missing scripts directory fails" 'no scripts/' -s "$repo" "$scratch/bad"
mkdir -p "$repo/scripts"
printf '# placeholder\n' > "$repo/scripts/readme.txt"
commit_version 2.0.11
reject "empty source selection fails" 'no .scrippo files' -s "$repo" "$scratch/bad"
git -C "$repo" -c user.name=Test -c user.email=test@example.com tag -a -m release v2.0.12
reject "annotated version tags resolve to commits" 'version must match' -p --tag v2.0.12 "$repo" "$scratch/bad"
write_version 3.0.0
mkdir -p "$repo/packages/nested"
printf '@info other 3.0.0\n@import "../../support.bash"\nother() -> "nested"\n' > "$repo/packages/nested/other.scrippo"
commit_version 3.0.0
check "multiple nested packages install" 0 "$(install_to -p "$repo" "$scratch/nested"; echo $?)"
check "nested output keeps relative package path" nested "$(bash -c 'source "$1"; other' _ "$scratch/nested/packages/example/content/nested/other.bash")"
write_version 3.0.1
printf '@info other 3.0.1\n@unknown\n' > "$repo/packages/nested/other.scrippo"
commit_version 3.0.1
reject "one bad package rejects whole repository" 'unknown macro' -p "$repo" "$scratch/bad"
check "multi-package failure publishes nothing" no "$([[ -e $scratch/bad/packages/example ]] && echo yes || echo no)"
printf '@info greet 3.0.2\n' > "$repo/scripts/greet.scrippo"
printf '%s\n' '-> missing' > "$repo/scripts/invalid.scrippo"
commit_version 3.0.2
reject "script metadata is required too" 'exactly one top-level @info' -s "$repo" "$scratch/bad-scripts"
check "temporary directories are cleaned after failures" "" "$(find "$scratch/bad" -name '.scrippo-install.*' -print)"

repo=$scratch/untagged
git init -q "$repo"
printf 'readme\n' > "$repo/README"
git -C "$repo" add .
git -C "$repo" -c user.name=Test -c user.email=test@example.com commit -qm init
reject "repositories must have version tags" 'no stable version tags' -p "$repo" "$scratch/no-tags"

(( failures == 0 ))
