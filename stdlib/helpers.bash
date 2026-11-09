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

function _scrippo_config_path {
  local id=$1 dir=$PWD
  case $id in
    "~/"*) id=$HOME/${id#"~/"} ;;
  esac
  if [[ $id == *.json ]]; then
    printf '%s\n' "$id"
    return
  fi
  if [[ ! $id =~ ^[A-Za-z0-9_.-]+$ ]]; then
    echo "config: invalid config id '$id' (use letters, digits, '_', '-', '.', or a path ending in .json)" >&2
    return 1
  fi
  while [[ $dir != / && $dir != "$HOME" ]]; do
    if [[ -d $dir/.scrippo ]]; then
      printf '%s\n' "$dir/.scrippo/configs/$id.config.json"
      return
    fi
    dir=$(dirname "$dir")
  done
  printf '%s\n' "$PWD/.scrippo/configs/$id.config.json"
}

# Print the value at a dotted key. Strings print raw, everything else as JSON.
# Returns 1 when the key is missing or null.
function _scrippo_config_get {
  local file=$_SCRIPPO_CONFIG_FILE
  [[ -f $file ]] || return 1
  jq -e -r --arg k "$1" '
    ($k | split(".") | map(if test("^[0-9]+$") then tonumber else . end)) as $p
    | (try getpath($p) catch null)
    | select(. != null)
    | if type == "string" then . else tojson end
  ' "$file" 2>/dev/null
}

# Print $1 (JSON) with the dotted key $2 set to $3. $3 is stored as JSON when it
# parses as JSON and as a string otherwise.
function _scrippo_config_put {
  jq --arg k "$2" --arg v "$3" '
    ($k | split(".") | map(if test("^[0-9]+$") then tonumber else . end)) as $p
    | ($v | try fromjson catch $v) as $x
    | setpath($p; $x)
  ' <<<"$1"
}

# Write JSON ($2) to the config file ($1), pretty printed.
function _scrippo_config_write {
  mkdir -p "$(dirname "$1")"
  printf '%s\n' "$2" | jq . > "$1.tmp.$$" && mv "$1.tmp.$$" "$1"
}

# Edit the selected config file through a menu: pick a field, then change it.
# Booleans are chosen from true/false, everything else is typed.
function _scrippo_config_interactive {
  local file=$_SCRIPPO_CONFIG_FILE json line key type value label choice i found
  local -a keys=() types=() values=() labels=()
  local done_label="Done"
  while true; do
    json='{}'
    [[ -f $file ]] && json=$(<"$file")
    keys=() types=() values=() labels=()
    while IFS=$'\t' read -r key type value; do
      keys+=("$key")
      types+=("$type")
      values+=("$value")
      labels+=("$key = $value")
    done < <(jq -r '
      [path(.. | select(type != "object" and type != "array")) as $p
       | [($p | map(tostring) | join(".")), (getpath($p) | type), (getpath($p) | tojson)]]
      | .[] | @tsv
    ' <<<"$json")
    if (( ${#keys[@]} == 0 )); then
      echo "config: nothing to edit in $file" >&2
      return 0
    fi
    _scrippo_select "Edit config" false "${labels[@]}" "$done_label"
    choice=$_scrippo_reply
    [[ -z $choice || $choice == "$done_label" ]] && return 0
    found=
    for i in "${!labels[@]}"; do
      if [[ ${labels[i]} == "$choice" ]]; then
        found=$i
        break
      fi
    done
    [[ -z $found ]] && return 0
    key=${keys[found]}
    type=${types[found]}
    value=${values[found]}
    if [[ $type == boolean ]]; then
      _scrippo_select "$key" false true false
      [[ -z $_scrippo_reply ]] && continue
      value=$_scrippo_reply
    else
      if [[ $type == string ]]; then
        value=$(jq -r . <<<"$value")
      fi
      _scrippo_input "$key [$value]"
      [[ -z $_scrippo_reply ]] && continue
      value=$_scrippo_reply
      if [[ $type == string ]]; then
        value=$(jq -c -n --arg v "$value" '$v')
      fi
    fi
    json=$(_scrippo_config_put "$json" "$key" "$value") || { echo "config: cannot set '$key'" >&2; return 1; }
    _scrippo_config_write "$file" "$json" || return 1
  done
}

function _scrippo_config {
  local force=$1 init=$2 interactive=$3 ndefaults=$4 count=$5
  shift 5
  local -a defaults=("${@:1:ndefaults}")
  local -a fields=("${@:ndefaults+1:count}")
  local -a assigns=("${@:ndefaults+count+1}")
  local field key value json sets=0 file i

  if ! command -v jq >/dev/null 2>&1; then
    echo "config: jq is required" >&2
    exit 1
  fi

  if [[ -n $init ]]; then
    file=$(_scrippo_config_path "$init") || exit 1
    if [[ $force == true || ! -e $file ]]; then
      mkdir -p "$(dirname "$file")" && printf '{}\n' > "$file" || exit 1
    elif ! jq -e . "$file" >/dev/null 2>&1; then
      echo "config: $file is not valid JSON (use --force to reset it)" >&2
      exit 1
    fi
    _SCRIPPO_CONFIG_FILE=$file
    if (( ndefaults % 2 != 0 )); then
      echo "config: -f expects a field and a value, got an odd number of words" >&2
      exit 1
    fi
    json=$(<"$file")
    for (( i = 0; i < ndefaults; i += 2 )); do
      key=${defaults[i]}
      if [[ -z $key ]]; then
        echo "config: missing field name after -f" >&2
        exit 1
      fi
      _scrippo_config_get "$key" >/dev/null && continue
      json=$(_scrippo_config_put "$json" "$key" "${defaults[i + 1]}") || { echo "config: cannot set '$key'" >&2; exit 1; }
      _scrippo_config_write "$file" "$json" || exit 1
    done
  elif (( ndefaults > 0 )); then
    echo "config: -f only works together with --init" >&2
    exit 1
  fi

  if [[ $interactive == true ]]; then
    if (( count > 0 || ${#assigns[@]} > 0 )); then
      echo "config: --interactive cannot be combined with a field or --assign" >&2
      exit 1
    fi
    if [[ -z ${_SCRIPPO_CONFIG_FILE-} ]]; then
      echo "config: no config file selected (run 'config --init <id|path>' first)" >&2
      exit 1
    fi
    _scrippo_config_interactive || exit 1
    return 0
  fi

  if (( count == 0 && ${#assigns[@]} == 0 )); then
    if [[ -z $init ]]; then
      echo "config: nothing to do (give a field to get or set, --init, --interactive, or --assign)" >&2
      exit 1
    fi
    return 0
  fi
  if [[ -z ${_SCRIPPO_CONFIG_FILE-} ]]; then
    echo "config: no config file selected (run 'config --init <id|path>' first)" >&2
    exit 1
  fi
  file=$_SCRIPPO_CONFIG_FILE

  for field in "${fields[@]}"; do
    [[ $field == *=* ]] && sets=$((sets + 1))
  done
  if (( sets > 0 && sets < count )); then
    echo "config: cannot get and set in the same call" >&2
    exit 1
  elif (( sets == 0 && count > 1 )); then
    echo "config: only one field can be read at a time" >&2
    exit 1
  fi

  if (( sets > 0 )); then
    json='{}'
    [[ -f $file ]] && json=$(<"$file")
    for field in "${fields[@]}"; do
      key=${field%%=*}
      value=${field#*=}
      if [[ -z $key ]]; then
        echo "config: missing field name in '$field'" >&2
        exit 1
      fi
      json=$(_scrippo_config_put "$json" "$key" "$value") || { echo "config: cannot set '$key'" >&2; exit 1; }
    done
    _scrippo_config_write "$file" "$json" || exit 1
  elif (( count == 1 )); then
    _scrippo_config_get "${fields[0]}" || return 1
  fi

  local item var
  for item in "${assigns[@]}"; do
    if [[ $item != *:* ]]; then
      echo "config: --assign expects field:VARIABLE, got '$item'" >&2
      exit 1
    fi
    key=${item%:*}
    var=${item##*:}
    if [[ ! $var =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
      echo "config: invalid variable name '$var' in --assign" >&2
      exit 1
    fi
    value=$(_scrippo_config_get "$key") || value=""
    printf -v "$var" '%s' "$value"
  done
}
