#!/usr/bin/env bash

function macro_import {
  local node=$1 path file tree nodes child content previous=$_compile_import_dir
  if (( $# != 2 )); then
    compile_error "$node" "@import: expected a single <path>"
  fi
  if ! compile_literal "$2"; then
    compile_error "$node" "@import: <path> must be a literal, received $2"
  fi
  path=$_compile_result
  [[ $path == /* ]] || path=$_compile_import_dir/$path
  if [[ ! -f $path ]]; then
    compile_error "$node" "@import: file not found: $path"
  fi
  file=$(cd "$(dirname "$path")" && pwd)/$(basename "$path")
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
