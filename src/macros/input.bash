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
        while (( $# > 0 )) && [[ $1 != -?* ]]; do
          options+=("$1")
          shift
        done
        ;;
      --multi)
        multi=true
        shift
        ;;
      --confirm)
        confirm=true
        shift
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
  elif [[ $select == false && $multi == true ]]; then
    compile_error "$node" "@input: --multi requires --select"
  elif [[ $select == true ]] && (( ${#options[@]} == 0 )); then
    compile_error "$node" "@input: --select requires at least one option"
  fi

  macro_input_runtime_answered
  if [[ $confirm == true ]]; then
    macro_input_runtime_key
    macro_input_runtime_confirm
    compile_emit "_scrippo_confirm $prompt"
    compile_emit "$var=\$_scrippo_reply"
  elif [[ $select == true ]]; then
    macro_input_runtime_key
    macro_input_runtime_select
    compile_emit "_scrippo_select $prompt $multi ${options[*]}"
    if [[ $multi == true ]]; then
      compile_emit "$var=(\"\${_scrippo_picks[@]}\")"
    else
      compile_emit "$var=\$_scrippo_reply"
    fi
  else
    macro_input_runtime_text
    compile_emit "_scrippo_input $prompt"
    compile_emit "$var=\$_scrippo_reply"
  fi
}

function macro_input_runtime_answered {
  compile_helper _scrippo_answered <<'EOF'
function _scrippo_answered {
  printf '\r\e[2K\e[32m✔\e[0m \e[1m%s\e[0m \e[2m·\e[0m \e[36m%s\e[0m\n' "$1" "$2" >&2
}
EOF
}

function macro_input_runtime_key {
  compile_helper _scrippo_key <<'EOF'
function _scrippo_key {
  local key rest
  IFS= read -rsn1 key
  if [[ $key == $'\e' ]]; then
    IFS= read -rsn2 -t 0.01 rest
    key+=$rest
  fi
  case $key in
    $'\e[A'|$'\eOA'|k) _scrippo_pressed=up ;;
    $'\e[B'|$'\eOB'|j) _scrippo_pressed=down ;;
    " ") _scrippo_pressed=space ;;
    "") _scrippo_pressed=enter ;;
    *) _scrippo_pressed=$key ;;
  esac
}
EOF
}

function macro_input_runtime_text {
  compile_helper _scrippo_input <<'EOF'
function _scrippo_input {
  read -rep $'\001\e[36m\002?\001\e[0;1m\002 '"$1"$'\001\e[0;2m\002 ›\001\e[0m\002 ' _scrippo_reply
  if [[ -t 0 && -t 2 ]]; then
    printf '\e[1A' >&2
    _scrippo_answered "$1" "$_scrippo_reply"
  fi
}
EOF
}

function macro_input_runtime_confirm {
  compile_helper _scrippo_confirm <<'EOF'
function _scrippo_confirm {
  local answer
  _scrippo_reply=false
  if [[ ! -t 0 || ! -t 2 ]]; then
    read -r answer
    [[ $answer == [yY]* ]] && _scrippo_reply=true
    return 0
  fi
  printf '\e[36m?\e[0m \e[1m%s\e[0m \e[2m(y/N)\e[0m ' "$1" >&2
  while true; do
    _scrippo_key
    case $_scrippo_pressed in
      y|Y) _scrippo_reply=true; break ;;
      n|N|enter) break ;;
    esac
  done
  answer=No
  [[ $_scrippo_reply == true ]] && answer=Yes
  _scrippo_answered "$1" "$answer"
}
EOF
}

function macro_input_runtime_select {
  compile_helper _scrippo_select <<'EOF'
function _scrippo_select {
  local prompt=$1 multi=$2 cursor=0 hint="↑↓ move · enter select" saved_trap line i
  local -a options=("${@:3}") picked=()
  local count=${#options[@]}
  _scrippo_reply=""
  _scrippo_picks=()
  if [[ ! -t 0 || ! -t 2 ]]; then
    _scrippo_select_plain "$@"
    return
  fi
  if [[ $multi == true ]]; then
    hint="↑↓ move · space toggle · a all · enter confirm"
  fi
  saved_trap=$(trap -p INT)
  trap 'printf "\e[?25h\n" >&2; exit 130' INT
  printf '\e[?25l\e[36m?\e[0m \e[1m%s\e[0m \e[2m%s\e[0m\n' "$prompt" "$hint" >&2
  while true; do
    for i in "${!options[@]}"; do
      line="  "
      if (( i == cursor )); then
        line=$'\e[36m❯\e[0m '
      fi
      if [[ $multi == true && -n ${picked[i]} ]]; then
        line+=$'\e[32m◉\e[0m '
      elif [[ $multi == true ]]; then
        line+=$'\e[2m◯\e[0m '
      fi
      if (( i == cursor )); then
        line+=$'\e[36m'"${options[i]}"$'\e[0m'
      else
        line+=${options[i]}
      fi
      printf '\r\e[2K%s\n' "$line" >&2
    done
    _scrippo_key
    case $_scrippo_pressed in
      up) cursor=$(( (cursor + count - 1) % count )) ;;
      down) cursor=$(( (cursor + 1) % count )) ;;
      space)
        if [[ -n ${picked[cursor]} ]]; then
          unset 'picked[cursor]'
        else
          picked[cursor]=1
        fi
        ;;
      a)
        if (( ${#picked[@]} == count )); then
          picked=()
        else
          for i in "${!options[@]}"; do
            picked[i]=1
          done
        fi
        ;;
      enter) break ;;
    esac
    printf '\e[%dA' "$count" >&2
  done
  printf '\e[%dA\r\e[J\e[?25h' "$(( count + 1 ))" >&2
  trap - INT
  eval "$saved_trap"
  if [[ $multi != true ]]; then
    _scrippo_reply=${options[cursor]}
    _scrippo_answered "$prompt" "$_scrippo_reply"
    return 0
  fi
  for i in "${!options[@]}"; do
    [[ -n ${picked[i]} ]] && _scrippo_picks+=("${options[i]}")
  done
  printf -v line '%s, ' "${_scrippo_picks[@]}"
  _scrippo_answered "$prompt" "${line%, }"
}

function _scrippo_select_plain {
  local prompt=$1 pick i
  local -a options=("${@:3}") picks=()
  for i in "${!options[@]}"; do
    printf '%d) %s\n' "$(( i + 1 ))" "${options[i]}" >&2
  done
  printf '%s\n' "$prompt" >&2
  IFS=', ' read -ra picks
  for pick in "${picks[@]}"; do
    if [[ $pick =~ ^[0-9]+$ ]] && (( pick >= 1 && pick <= ${#options[@]} )); then
      _scrippo_picks+=("${options[pick - 1]}")
    fi
  done
  _scrippo_reply=${_scrippo_picks[0]}
}
EOF
}
