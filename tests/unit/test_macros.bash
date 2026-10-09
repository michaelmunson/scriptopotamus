#!/usr/bin/env bash

source "$(dirname "$0")/test_utils.bash"

root=$(cd "$SCRIPT_DIR/../.." && pwd)
scrippo=$root/scrippo
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/project/sub" "$scratch/elsewhere"
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

cat > "$project/sub/rootdir.scrippo" <<'EOF'
.statement()
	@rootdir

.arrow()
	-> @rootdir

.assign()
	dir<str>=@rootdir
	-> "$dir"

.path()
	cd /
	-> "@rootdir/rootdir.scrippo"

.untouched()
	arr=(a b)
	-> "${arr[@]} user@rootdir.com @rootdirs"

.enter()
	cd /
	@rootdir --enter
	pwd
EOF

cat > "$project/sub/rootdir_args.scrippo" <<'EOF'
.args()
	@rootdir extra
EOF

function run_from {
  local dir=$1
  shift
  (cd "$dir" && "$scrippo" "$@")
}

script=../project/sub/rootdir.scrippo
check "@rootdir statement prints the script dir" "$project/sub" "$(run_from "$scratch/elsewhere" "$script" statement)"
check "@rootdir in echo" "$project/sub" "$(run_from "$scratch/elsewhere" "$script" arrow)"
check "@rootdir in typed assignment" "$project/sub" "$(run_from "$scratch/elsewhere" "$script" assign)"
check "@rootdir survives cd" "$project/sub/rootdir.scrippo" "$(run_from "$scratch/elsewhere" "$script" path)"
check "@rootdir from script dir" "$project/sub" "$(run_from "$project/sub" rootdir.scrippo arrow)"
check "@rootdir leaves other @ text alone" "a b user@rootdir.com @rootdirs" "$(run_from "$scratch/elsewhere" "$script" untouched)"
check "@rootdir --enter changes into the script dir" "$project/sub" "$(run_from "$scratch/elsewhere" "$script" enter)"
check "@rootdir rejects other arguments" "compile: line 2: @rootdir: unexpected 'extra', expected [--enter]" "$(run_from "$project/sub" rootdir_args.scrippo args 2>&1)"

(( failures == 0 ))
