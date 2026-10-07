#!/usr/bin/env bash

function macro_input {
  local node=$1 prompt="" var="" select=false multi=false confirm=false options=()
  local name_re='^[a-zA-Z_][a-zA-Z0-9_]*$'
  shift
  while (( $# > 0 )); do
    case $1 in
      -v)
        if (( $# < 2 )); then
          compile_error "$node" "@input: -v requires a value"
        fi
        compile_literal "$2"
        var=$_compile_result
        shift 2
        ;;
      -s|--select)
        select=true
        shift
        ;;
      --multi)
        multi=true
        shift
        ;;
      --confirm)
        confirm=true
        shift
        ;;
      --options)
        shift
        while (( $# > 0 )) && [[ $1 != -?* ]]; do
          options+=("$1")
          shift
        done
        ;;
      -?*)
        compile_error "$node" "@input: unknown option '$1'"
        ;;
      *)
        if [[ -n $prompt ]]; then
          compile_error "$node" "@input: unexpected '$1'"
        fi
        prompt=$1
        shift
        ;;
    esac
  done

  if [[ -z $prompt ]]; then
    compile_error "$node" "@input: <prompt> is required"
  elif [[ -z $var ]]; then
    compile_error "$node" "@input: -v is required"
  elif [[ ! $var =~ $name_re ]]; then
    compile_error "$node" "@input: -v expected a variable name, received '$var'"
  elif [[ $select == true && $confirm == true ]]; then
    compile_error "$node" "@input: --select and --confirm are mutually exclusive"
  elif [[ $select == false && ( $multi == true || ${#options[@]} -gt 0 ) ]]; then
    compile_error "$node" "@input: --options and --multi require --select"
  elif [[ $select == true ]] && (( ${#options[@]} == 0 )); then
    compile_error "$node" "@input: --select requires --options"
  fi

  if [[ $confirm == true ]]; then
    macro_input_confirm "$prompt" "$var"
  elif [[ $multi == true ]]; then
    macro_input_multi "$prompt" "$var" "${options[@]}"
  elif [[ $select == true ]]; then
    macro_input_select "$prompt" "$var" "${options[@]}"
  else
    compile_emit "read -rp $prompt' ' $var"
  fi
}

function macro_input_confirm {
  local prompt=$1 var=$2
  compile_emit "read -rp $prompt' [y/N] ' _scrippo_reply"
  compile_emit "if [[ \$_scrippo_reply == [yY]* ]]; then"
  compile_emit "  $var=true"
  compile_emit "else"
  compile_emit "  $var=false"
  compile_emit "fi"
}

function macro_input_select {
  local prompt=$1 var=$2
  shift 2
  compile_emit "PS3=$prompt' '"
  compile_emit "select $var in $*; do"
  compile_emit "  [[ -n \$$var ]] && break"
  compile_emit "done"
}

function macro_input_multi {
  local prompt=$1 var=$2
  shift 2
  compile_emit "_scrippo_options=($*)"
  compile_emit "for _scrippo_i in \"\${!_scrippo_options[@]}\"; do"
  compile_emit "  echo \"\$(( _scrippo_i + 1 ))) \${_scrippo_options[_scrippo_i]}\" >&2"
  compile_emit "done"
  compile_emit "IFS=', ' read -rp $prompt' ' -a _scrippo_picks"
  compile_emit "$var=()"
  compile_emit "for _scrippo_i in \"\${_scrippo_picks[@]}\"; do"
  compile_emit "  if [[ \$_scrippo_i =~ ^[0-9]+\$ ]] && (( _scrippo_i >= 1 && _scrippo_i <= \${#_scrippo_options[@]} )); then"
  compile_emit "    $var+=(\"\${_scrippo_options[_scrippo_i - 1]}\")"
  compile_emit "  fi"
  compile_emit "done"
}
