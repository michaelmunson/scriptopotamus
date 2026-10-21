#!/usr/bin/env bash

function macro_import_remember {
  local file
  [[ -f $1 ]] || return 0
  file=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
  _compile_imported[$file]=1
}

function macro_import {
  local node=$1 pattern
  if (( $# != 2 )); then
    compile_error "$node" "@import: expected a single <path>"
  fi
  if ! macro_import_literal "$2"; then
    compile_error "$node" "@import: <path> must be a literal, received $2"
  fi
  pattern=$_compile_result
  [[ $pattern == /* ]] || pattern=$_compile_import_dir/$pattern
  macro_import_sandbox "$node" "$pattern"
  if [[ $pattern == *[*?\[]* ]]; then
    macro_import_glob "$node" "$pattern"
  else
    macro_import_file "$node" "$pattern"
  fi
}

# Check the lexical path before expanding globs or touching the filesystem.
function macro_import_sandbox {
  [[ -n ${SCRIPPO_SANDBOX_ROOT:-} ]] || return 0
  local node=$1 path=$2 component normalized IFS=/
  local -a components=() parts=()
  read -r -a components <<< "$path"
  for component in "${components[@]}"; do
    case $component in
      ''|.) ;;
      ..) (( ${#parts[@]} == 0 )) || unset "parts[$(( ${#parts[@]} - 1 ))]" ;;
      *) parts+=("$component") ;;
    esac
  done
  normalized=/${parts[*]}
  if [[ $normalized != "$SCRIPPO_SANDBOX_ROOT/"* ]]; then
    compile_error "$node" '@import: path is outside the package sandbox'
  fi
}

function macro_import_literal {
  local raw=$1
  local path_re='^[][A-Za-z0-9_./+*?-]+$'
  if compile_literal "$raw"; then
    return 0
  fi
  [[ $raw =~ $path_re ]] || return 1
  _compile_result=$raw
}

function macro_import_glob {
  local node=$1 pattern=$2 file
  local -a files=()
  while IFS= read -r -d '' file; do
    [[ -f $file ]] && files+=("$file")
  done < <(
    IFS=
    shopt -s nullglob globstar
    shopt -u failglob
    for file in $pattern; do
      printf '%s\0' "$file"
    done
  )
  (( ${#files[@]} )) || compile_error "$node" "@import: no matches: $pattern"
  for file in "${files[@]}"; do
    macro_import_file "$node" "$file"
  done
}

function macro_import_file {
  local node=$1 path=$2 file tree nodes child content previous=$_compile_import_dir
  if [[ ! -f $path ]]; then
    compile_error "$node" "@import: file not found: $path"
  fi
  file=$(cd "$(dirname "$path")" && pwd)/$(basename "$path")
  if [[ -n ${SCRIPPO_SANDBOX_ROOT:-} ]]; then
    local physical
    physical=$(cd -P "$(dirname "$path")" && pwd)/$(basename "$path")
    if [[ -L $path || $physical != "$SCRIPPO_SANDBOX_ROOT/"* ]]; then
      compile_error "$node" '@import: path is outside the package sandbox'
    fi
  fi
  [[ -z ${_compile_imported[$file]} ]] || return 0
  _compile_imported[$file]=1

  if [[ $file == *.scrippo ]]; then
    tree=$(ast "$file") || exit 1
    mapfile -t nodes < <(jq -c '.body[]' <<< "$tree")
    _compile_import_dir=$(dirname "$file")
    for child in "${nodes[@]}"; do
      compile_node "$child"
    done
    _compile_import_dir=$previous
  else
    content=$(< "$file")
    compile_emit "${content#\#!*$'\n'}"
  fi
}
