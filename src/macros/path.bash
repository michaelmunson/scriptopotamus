#!/usr/bin/env bash

function macro_rootdir {
  local node=$1
  macro_rootdir_runtime
  if (( $# == 1 )); then
    compile_emit "echo \"\$_scrippo_rootdir\""
  elif (( $# == 2 )) && [[ $2 == --enter ]]; then
    compile_emit "cd \"\$_scrippo_rootdir\""
  else
    compile_error "$node" "@rootdir: unexpected '${*:2}', expected [--enter]"
  fi
}

function macro_rootdir_expand {
  local text=$1
  local rootdir_re='(^|[^a-zA-Z0-9_$])@rootdir([^a-zA-Z0-9_]|$)'
  while [[ $text =~ $rootdir_re ]]; do
    text=${text/"${BASH_REMATCH[0]}"/"${BASH_REMATCH[1]}\$_scrippo_rootdir${BASH_REMATCH[2]}"}
    macro_rootdir_runtime
  done
  _compile_result=$text
}

function macro_rootdir_runtime {
  compile_helper _scrippo_rootdir <<'EOF'
_scrippo_rootdir=$(cd "$(dirname "$0")" && pwd)
EOF
}
