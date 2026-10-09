#!/usr/bin/env bash

source "$(dirname "$0")/test_utils.bash"

root=$(cd "$SCRIPT_DIR/../.." && pwd)
scrippo=$root/scrippo
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
cp -R "$root/tests/fixtures/shell/project" "$scratch/project"
mkdir "$scratch/project/.git" "$scratch/home"
project=$(cd "$scratch/project" && pwd)
failures=0

function check {
  local name=$1 expected=$2 actual=$3
  if expect_equal "$expected" "$actual"; then
    echo "ok: $name"
  else
    echo "FAIL: $name"
    failures=$(( failures + 1 ))
  fi
}

function complete_in {
  local dir=$1
  shift
  (cd "$dir" && "$scrippo" __complete -- "$@")
}

check "aliases at project root" \
  "$(printf 'app\t%s\t\nbuild\t%s\tbuild' "$project/app.scrippo" "$project/.scrippo")" \
  "$("$scrippo" __aliases "$project")"

check "aliases in subdirectory keep parent aliases" \
  "$(printf 'test\t%s\t\napp\t%s\t\nbuild\t%s\tbuild' "$project/tests/test.scrippo" "$project/app.scrippo" "$project/.scrippo")" \
  "$("$scrippo" __aliases "$project/tests")"

check "no aliases outside the project" "" "$("$scrippo" __aliases "$scratch")"

check "complete top-level command" "run" "$(complete_in "$project" app.scrippo r)"
check "complete nested command" "dev" "$(complete_in "$project" app.scrippo run d)"
check "complete nested command from empty word" "dev" "$(complete_in "$project" app.scrippo run "")"
check "no completion past a leaf command" "" "$(complete_in "$project" app.scrippo run dev "")"
check "complete .scrippo commands" "build" "$(complete_in "$project" .scrippo "")"
check "complete flags" "--flag" "$(complete_in "$project" .scrippo build --)"
check "complete without a file" "$(printf 'configure\napp.scrippo\n.scrippo')" "$(complete_in "$project" "" "")"

printf '@import "tests/test.scrippo"\n.own()\n\t-> "own"\n' > "$project/imports.scrippo"
check "complete imported commands" "$(printf 'own\ntest')" "$(complete_in "$project" imports.scrippo "")"

check "source is empty when disabled" "_scrippo_alias_pwd=" "$(HOME=$scratch/home "$scrippo" __source)"

HOME=$scratch/home "$scrippo" configure --autoload 1 --autocomplete 1 > /dev/null 2>&1
check "configure writes config" '{"autoload":1,"autocomplete":1}' "$(jq -c . "$scratch/home/.scriptopotamus/config.json")"

source=$(HOME=$scratch/home "$scrippo" __source)
check "source includes autoload" "yes" "$([[ $source == *_scrippo_refresh* ]] && echo yes)"
check "source includes autocomplete" "yes" "$([[ $source == *_scrippo_complete_bash* ]] && echo yes)"

HOME=$scratch/home "$scrippo" configure --autocomplete 0 > /dev/null 2>&1
check "configure keeps unspecified keys" '{"autoload":1,"autocomplete":0}' "$(jq -c . "$scratch/home/.scriptopotamus/config.json")"

check "configure rejects invalid values" "1" "$(HOME=$scratch/home "$scrippo" configure --autoload 2 > /dev/null 2>&1; echo $?)"

(( failures == 0 ))
