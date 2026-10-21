#!/usr/bin/env bash

# Metadata is parsed as literals, never evaluated or emitted as shell code.
function macro_info {
  local node=$1 arg
  shift
  if (( $# < 2 || $# > 3 )); then
    compile_error "$node" '@info: expected <name> <version> [<description>]'
  fi
  local -a values=()
  for arg in "$@"; do
    compile_literal "$arg" || compile_error "$node" '@info: values must be literals'
    values+=("$_compile_result")
  done
  [[ ${values[0]} =~ ^[a-zA-Z_][a-zA-Z0-9_.-]*$ ]] || compile_error "$node" '@info: invalid name'
  package_version_valid "${values[1]}" || compile_error "$node" '@info: expected a version such as 1.2.3'
  _scrippo_info_name=${values[0]}
  _scrippo_info_version=${values[1]}
  _scrippo_info_description=${values[2]:-}
}

function package_version_valid {
  # Stable releases only; a repository tag may additionally have a leading v.
  [[ $1 =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]
}
