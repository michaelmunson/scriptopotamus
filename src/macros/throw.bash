#!/usr/bin/env bash

function macro_throw {
  local node=$1 code=$2
  local code_re='^([0-9]+|\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?)$'
  if (( $# < 3 )); then
    compile_error "$node" "@throw: <code> <msg> required"
  fi
  if [[ ! $code =~ $code_re ]]; then
    compile_error "$node" "@throw: <code> expected int, received $code"
  fi
  macro_prt_style "${_macro_prt_codes[red]}" "${*:3}"
  compile_emit "echo $_compile_result >&2"
  compile_emit "exit $code"
}
