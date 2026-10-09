#!/usr/bin/env bash

declare -gA _macro_prt_codes=(
  [bold]=1
  [dim]=2
  [italic]=3
  [underline]=4
  [black]=30
  [red]=31
  [green]=32
  [yellow]=33
  [blue]=34
  [magenta]=35
  [cyan]=36
  [white]=37
)

function macro_prt {
  local node=$1 flags="" join=" " text="" style="" piece codes=() pieces=()
  shift
  while (( $# > 0 )); do
    case $1 in
      -s|--style)
        if ! macro_prt_code "$2"; then
          compile_error "$node" "@prt: unknown style '$2'"
        fi
        codes=("$_compile_result")
        shift 2
        while (( $# > 0 )) && macro_prt_code "$1"; do
          codes+=("$_compile_result")
          shift
        done
        printf -v style '%s;' "${codes[@]}"
        ;;
      -j|--join)
        if (( $# < 2 )); then
          compile_error "$node" "@prt: $1 requires a value"
        fi
        join=$2
        shift 2
        ;;
      -n)
        flags=" -n"
        shift
        ;;
      *)
        macro_prt_style "${style%;}" "$1"
        pieces+=("$_compile_result")
        style=""
        shift
        ;;
    esac
  done
  if [[ -n $style ]]; then
    compile_error "$node" "@prt: -s expected a message after the style"
  fi
  for piece in "${pieces[@]}"; do
    text+=${text:+$join}$piece
  done
  compile_emit "echo$flags${text:+ $text}"
}

function macro_prt_code {
  compile_literal "$1" || return 1
  [[ -n $_compile_result ]] || return 1
  _compile_result=${_macro_prt_codes[$_compile_result]}
  [[ -n $_compile_result ]]
}

function macro_prt_style {
  local codes=$1 text=$2
  if [[ -n $codes ]]; then
    text="\$'\\e[${codes}m'$text\$'\\e[0m'"
  fi
  _compile_result=$text
}
