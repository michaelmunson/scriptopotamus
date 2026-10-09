#!/usr/bin/env bash

_shell_name_re='^[a-zA-Z_][a-zA-Z0-9_.-]*$'

function shell_config_read {
  local file=$HOME/.scriptopotamus/config.json
  if [[ -f $file ]]; then
    jq -c '{autoload: 0, autocomplete: 0} + .' "$file"
  else
    echo '{"autoload":0,"autocomplete":0}'
  fi
}

function shell_configure {
  local autoload=$1 autocomplete=$2 dir=$HOME/.scriptopotamus config
  config=$(shell_config_read) || return 1
  if [[ -n $autoload ]]; then
    config=$(jq -c --argjson value "$autoload" '.autoload = $value' <<< "$config")
  fi
  if [[ -n $autocomplete ]]; then
    config=$(jq -c --argjson value "$autocomplete" '.autocomplete = $value' <<< "$config")
  fi
  mkdir -p "$dir"
  jq . <<< "$config" > "$dir/config.json"
  cat "$dir/config.json"
  echo "restart your shell or run: source <(scrippo __source)" >&2
}

function shell_enabled {
  [[ $(shell_config_read | jq -r ".$1") == 1 ]]
}

function shell_scope_dirs {
  local start=${1:-$PWD} current
  start=$(cd "$start" && pwd) || return 1
  current=$start
  _shell_dirs=()
  while true; do
    _shell_dirs+=("$current")
    [[ $current == / || $current == "$HOME" ]] && return 0
    current=$(dirname "$current")
  done
}

function shell_tree {
  local -A visited=()
  shell_tree_collect "$1" | jq -sc 'add // []'
}

function shell_tree_collect {
  local file tree pattern match
  [[ -f $1 ]] || return 0
  file=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
  [[ -z ${visited[$file]} ]] || return 0
  visited[$file]=1
  tree=$(ast "$file" 2>/dev/null) || return 0
  jq -c '[.body[] | select(.type == "command")]' <<< "$tree"
  while IFS= read -r pattern; do
    pattern=${pattern#[\"\']}
    pattern=${pattern%[\"\']}
    [[ $pattern == /* ]] || pattern=$(dirname "$file")/$pattern
    while IFS= read -r -d '' match; do
      [[ $match == *.scrippo ]] && shell_tree_collect "$match"
    done < <(
      IFS=
      shopt -s nullglob globstar
      for match in $pattern; do
        printf '%s\0' "$match"
      done
    )
  done < <(jq -r '.body[] | select(.type == "macro" and .name == "import") | .args[0] // empty' <<< "$tree")
}

function shell_alias_add {
  [[ $1 =~ $_shell_name_re && -z ${seen[$1]} ]] || return 0
  seen[$1]=1
  printf '%s\t%s\t%s\n' "$1" "$2" "$3"
}

function shell_aliases {
  local dir file name
  local -A seen=()
  shell_scope_dirs "$1" || return 1
  for dir in "${_shell_dirs[@]}"; do
    for file in "$dir"/*.scrippo; do
      [[ -f $file ]] || continue
      name=${file##*/}
      shell_alias_add "${name%.scrippo}" "$file" ""
    done
    file=$dir/.scrippo
    [[ -f $file ]] || continue
    while IFS= read -r name; do
      shell_alias_add "$name" "$file" "$name"
    done < <(shell_tree "$file" | jq -r '.[] | select(.name != "") | .name')
  done
}

function shell_complete {
  local file=$1 current="" node next word candidate descend=true
  local -a words=() candidates=()
  if (( $# >= 2 )); then
    current=${!#}
    words=("${@:2:$# - 2}")
  fi

  if [[ ! -f $file ]]; then
    candidates=(configure)
    for candidate in *.scrippo .scrippo; do
      [[ -f $candidate ]] && candidates+=("$candidate")
    done
  else
    node=$(shell_tree "$file" | jq -c '{body: ., params: [.[] | select(.name == "") | .params[]]}')
    for word in "${words[@]}"; do
      [[ $word == -* ]] && continue
      next=$(jq -c --arg name "$word" 'first(.body[] | select(.type == "command" and .name == $name and $name != "")) // empty' <<< "$node")
      if [[ -z $next ]]; then
        descend=false
        break
      fi
      node=$next
    done
    if [[ $current == -* ]]; then
      mapfile -t candidates < <(jq -r "$_compile_param_jq"'.params | leaves | select(.kind == "opt") | .flag, (.aliases // [])[]' <<< "$node")
    elif [[ $descend == true ]]; then
      mapfile -t candidates < <(jq -r '.body[] | select(.type == "command" and .name != "") | .name' <<< "$node")
    fi
  fi

  for candidate in "${candidates[@]}"; do
    [[ $candidate == "$current"* ]] && printf '%s\n' "$candidate"
  done
}

function shell_source {
  echo "_scrippo_alias_pwd="
  if shell_enabled autocomplete; then
    shell_snippet_autocomplete
  fi
  if shell_enabled autoload; then
    shell_snippet_autoload
  fi
}

function shell_snippet_autocomplete {
  cat <<'EOF'
_scrippo_autocomplete=1

_scrippo_lookup() {
  local entry name file command
  _scrippo_file=
  _scrippo_command=
  for entry in "${_scrippo_aliases[@]}"; do
    IFS=$'\t' read -r name file command <<< "$entry"
    if [ "$name" = "$1" ]; then
      _scrippo_file=$file
      _scrippo_command=$command
      return 0
    fi
  done
  return 1
}

_scrippo_candidates() {
  local line
  _scrippo_found=()
  while IFS= read -r line; do
    [ -n "$line" ] && _scrippo_found+=("$line")
  done < <(scrippo __complete -- "$@" 2>/dev/null)
}

_scrippo_complete_bash() {
  local cur=${COMP_WORDS[COMP_CWORD]} file i=2
  local -a args
  if [ "$1" = scrippo ]; then
    file=
    [ "$COMP_CWORD" -gt 1 ] && file=${COMP_WORDS[1]}
  else
    _scrippo_lookup "$1" || return 0
    file=$_scrippo_file
    [ -n "$_scrippo_command" ] && args+=("$_scrippo_command")
    i=1
  fi
  while [ "$i" -lt "$COMP_CWORD" ]; do
    args+=("${COMP_WORDS[i]}")
    i=$((i + 1))
  done
  _scrippo_candidates "$file" "${args[@]}" "$cur"
  COMPREPLY=("${_scrippo_found[@]}")
}

_scrippo_complete_zsh() {
  local file i=3
  local -a args
  if [ "${words[1]}" = scrippo ]; then
    file=
    [ "$CURRENT" -gt 2 ] && file=${(Q)words[2]}
  else
    _scrippo_lookup "${words[1]}" || return 1
    file=$_scrippo_file
    [ -n "$_scrippo_command" ] && args+=("$_scrippo_command")
    i=2
  fi
  while [ "$i" -lt "$CURRENT" ]; do
    args+=("${words[$i]}")
    i=$((i + 1))
  done
  _scrippo_candidates "$file" "${args[@]}" "${words[$CURRENT]}"
  if [ "${#_scrippo_found[@]}" -gt 0 ]; then
    compadd -- "${_scrippo_found[@]}"
  elif [ "$CURRENT" -eq 2 ]; then
    _files
  fi
}

_scrippo_complete_register() {
  if [ -n "$ZSH_VERSION" ]; then
    whence compdef >/dev/null 2>&1 && compdef _scrippo_complete_zsh "$1"
  else
    complete -o default -F _scrippo_complete_bash "$1"
  fi
}

_scrippo_complete_register scrippo
EOF
}

function shell_snippet_autoload {
  cat <<'EOF'
_scrippo_taken() {
  if [ -n "$ZSH_VERSION" ]; then
    case $(whence -w "$1") in
      *": alias"|*": function") return 0 ;;
    esac
  else
    case $(type -t "$1") in
      alias|function) return 0 ;;
    esac
  fi
  return 1
}

_scrippo_refresh() {
  local entry name file command
  [ "$PWD" = "$_scrippo_alias_pwd" ] && return 0
  _scrippo_alias_pwd=$PWD
  for entry in "${_scrippo_aliases[@]}"; do
    IFS=$'\t' read -r name file command <<< "$entry"
    unalias "$name" 2>/dev/null
    [ -z "$ZSH_VERSION" ] && complete -r "$name" 2>/dev/null
  done
  _scrippo_aliases=()
  while IFS=$'\t' read -r name file command; do
    [ -n "$name" ] || continue
    _scrippo_taken "$name" && continue
    alias "$name=scrippo $(printf '%q' "$file")${command:+ $command}"
    _scrippo_aliases+=("$name"$'\t'"$file"$'\t'"$command")
    [ -n "$_scrippo_autocomplete" ] && _scrippo_complete_register "$name"
  done < <(scrippo __aliases "$PWD" 2>/dev/null)
  return 0
}

if [ -n "$ZSH_VERSION" ]; then
  autoload -Uz add-zsh-hook
  add-zsh-hook chpwd _scrippo_refresh
else
  case ";$PROMPT_COMMAND;" in
    *";_scrippo_refresh;"*) ;;
    *) PROMPT_COMMAND="_scrippo_refresh${PROMPT_COMMAND:+;$PROMPT_COMMAND}" ;;
  esac
fi

_scrippo_refresh
EOF
}
