function _scrippo_prt {
  local -A codes=([bold]=1 [dim]=2 [italic]=3 [underline]=4 [black]=30 [red]=31 [green]=32 [yellow]=33 [blue]=34 [magenta]=35 [cyan]=36 [white]=37)
  local join=" " newline=true style="" piece
  local -a pieces=() codes_list=()
  while (( $# > 0 )); do
    case $1 in
      -s|--style)
        shift
        codes_list=()
        while (( $# > 0 )) && [[ -n ${codes[$1]+x} ]]; do
          codes_list+=("${codes[$1]}")
          shift
        done
        local IFS=$';'
        style="${codes_list[*]}"
        IFS=$' \t\n'
        ;;
      -j|--join)
        join=$2
        shift 2
        ;;
      -n)
        newline=false
        shift
        ;;
      --)
        shift
        break
        ;;
      -?*)
        echo "prt: unknown option '$1'" >&2
        exit 1
        ;;
      *)
        if [[ -n $style ]]; then
          pieces+=($'\e['"$style"$'m'"$1"$'\e[0m')
          style=""
        else
          pieces+=("$1")
        fi
        shift
        ;;
    esac
  done
  while (( $# > 0 )); do
    pieces+=("$1")
    shift
  done
  local IFS=$join
  piece="${pieces[*]}"
  if [[ $newline == true ]]; then
    printf '%s\n' "$piece"
  else
    printf '%s' "$piece"
  fi
}

function _scrippo_answered {
  printf '\r\e[2K\e[32m✔\e[0m \e[1m%s\e[0m \e[2m·\e[0m \e[36m%s\e[0m\n' "$1" "$2" >&2
}

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

function _scrippo_input {
  read -rep $'\001\e[36m\002?\001\e[0;1m\002 '"$1"$'\001\e[0;2m\002 ›\001\e[0m\002 ' _scrippo_reply
  if [[ -t 0 && -t 2 ]]; then
    printf '\e[1A' >&2
    _scrippo_answered "$1" "$_scrippo_reply"
  fi
}

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
      if [[ $multi == true && -n ${picked[i]:-} ]]; then
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
        if [[ -n ${picked[cursor]:-} ]]; then
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
    [[ -n ${picked[i]:-} ]] && _scrippo_picks+=("${options[i]}")
  done
  printf -v line '%s, ' "${_scrippo_picks[@]+"${_scrippo_picks[@]}"}"
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
  for pick in "${picks[@]+"${picks[@]}"}"; do
    if [[ $pick =~ ^[0-9]+$ ]] && (( pick >= 1 && pick <= ${#options[@]} )); then
      _scrippo_picks+=("${options[pick - 1]}")
    fi
  done
  _scrippo_reply=${_scrippo_picks[0]:-}
}
