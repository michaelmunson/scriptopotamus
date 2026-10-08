#!/usr/bin/env bash

find_bash4() {
  local dir candidate
  local dirs
  IFS=':' read -r -a dirs <<< "$PATH"
  dirs+=(/opt/homebrew/bin /usr/local/bin)
  for dir in "${dirs[@]}"; do
    [[ -n $dir ]] || continue
    candidate=$dir/bash
    if [[ -x $candidate ]] && "$candidate" -c '(( BASH_VERSINFO[0] >= 4 ))'; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

if (( BASH_VERSINFO[0] < 4 )); then
  if [[ -f ${BASH_SOURCE[0]} ]]; then
    newer=$(find_bash4) || {
      echo "install: bash 4 or newer is required" >&2
      exit 1
    }
    exec "$newer" "${BASH_SOURCE[0]}" "$@"
  fi
  echo "install: bash 4 or newer is required" >&2
  exit 1
fi

set -euo pipefail

prefix=${PREFIX:-/usr/local}
source_url=https://github.com/michaelmunson/scriptopotamus/archive/refs/heads/main.tar.gz

while (( $# > 0 )); do
  case $1 in
    --prefix)
      if [[ -z ${2:-} ]]; then
        echo "install: --prefix requires a path" >&2
        exit 1
      fi
      prefix=$2
      shift 2
      ;;
    --prefix=*)
      prefix=${1#--prefix=}
      if [[ -z $prefix ]]; then
        echo "install: --prefix requires a path" >&2
        exit 1
      fi
      shift
      ;;
    -h|--help)
      cat <<EOF
usage: install.sh [--prefix DIR]

Builds scrippo and installs it to DIR/bin.
DIR defaults to /usr/local, or to PREFIX when that is set.
Requires bash 4 or newer, jq, and bc.
EOF
      exit 0
      ;;
    *)
      echo "install: unknown argument '$1'" >&2
      exit 1
      ;;
  esac
done

for tool in jq bc; do
  if ! command -v "$tool" >/dev/null; then
    echo "install: $tool is required" >&2
    exit 1
  fi
done

root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
if [[ ! -f $root/src/compiler.bash || ! -f $root/src/scrippo.scrippo ]]; then
  if ! command -v curl >/dev/null || ! command -v tar >/dev/null; then
    echo "install: curl and tar are required" >&2
    exit 1
  fi
  root=$scratch/src
  mkdir -p "$root"
  curl -fsSL "$source_url" | tar -xz -C "$root" --strip-components=1
fi

built=$scratch/scrippo
{
  printf '#!%s\n' "$BASH"
  "$BASH" "$root/src/compiler.bash" "$root/src/scrippo.scrippo" | tail -n +2
} > "$built"

bindir=$prefix/bin
parent=$(dirname "$bindir")
if [[ -d $bindir && -w $bindir ]] || [[ ! -e $bindir && -w $parent ]]; then
  mkdir -p "$bindir"
  install -m 755 "$built" "$bindir/scrippo"
else
  sudo mkdir -p "$bindir"
  sudo install -m 755 "$built" "$bindir/scrippo"
fi

echo "installed $bindir/scrippo"
if [[ ":$PATH:" != *":$bindir:"* ]]; then
  echo "install: $bindir is not on PATH" >&2
fi
