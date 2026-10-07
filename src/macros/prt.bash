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
  local node=$1 flags="" style codes=() msg=()
  shift
  while (( $# > 0 )); do
    case $1 in
      -s|--style)
        if ! macro_prt_code "$2"; then
          compile_error "$node" "@prt: unknown style '$2'"
        fi
        codes+=("$_compile_result")
        shift 2
        while (( $# > 0 )) && macro_prt_code "$1"; do
          codes+=("$_compile_result")
          shift
        done
        ;;
      -n)
        flags=" -n"
        shift
        ;;
      *)
        msg+=("$1")
        shift
        ;;
    esac
  done
  printf -v style '%s;' "${codes[@]}"
  macro_prt_echo "$flags" "${style%;}" "${msg[@]}"
  compile_emit "$_compile_result"
}

function macro_prt_code {
  compile_literal "$1" || return 1
  [[ -n $_compile_result ]] || return 1
  _compile_result=${_macro_prt_codes[$_compile_result]}
  [[ -n $_compile_result ]]
}

function macro_prt_echo {
  local flags=$1 codes=$2 text
  shift 2
  text="$*"
  if [[ -n $codes ]]; then
    text="\$'\\e[${codes}m'$text\$'\\e[0m'"
  fi
  _compile_result="echo$flags $text"
}
