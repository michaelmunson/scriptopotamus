#!/usr/bin/env bash

function compile_macro {
  local node=$1 name raw_args arg args=()
  name=$(jq -r .name <<< "$node")
  if ! declare -F "macro_$name" > /dev/null; then
    compile_error "$node" "unknown macro '@$name'"
  fi
  mapfile -t raw_args < <(jq -r '.args[]' <<< "$node")
  for arg in "${raw_args[@]}"; do
    compile_expr "$arg"
    args+=("$_compile_result")
  done
  "macro_$name" "$node" "${args[@]}"
}
