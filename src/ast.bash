#!/usr/bin/env bash

function ast {
  local path=${1:?"ast: <path> Required"}
  mapfile -t _ast_lines < "$path" || return 1
  _ast_idx=0
  _ast_last=0
  ast_block -1
  jq '{type: "program", body: .}' <<< "$_ast_result"
}

function ast_error {
  echo "ast: line $(( _ast_idx + 1 )): $1" >&2
  exit 1
}

function ast_next {
  _ast_idx=$(( _ast_idx + 1 ))
  _ast_last=$_ast_idx
}

function ast_peek {
  local line
  while (( _ast_idx < ${#_ast_lines[@]} )); do
    line=${_ast_lines[_ast_idx]}
    _ast_text=${line#"${line%%[![:space:]]*}"}
    _ast_text=${_ast_text%"${_ast_text##*[![:space:]]}"}
    _ast_indent=$(( ${#line} - ${#_ast_text} ))
    if [[ -n $_ast_text && $_ast_text != \#* ]]; then
      return 0
    fi
    _ast_idx=$(( _ast_idx + 1 ))
  done
  return 1
}

function ast_list {
  _ast_result=$(printf '%s\n' "$@" | jq -sc .)
}

function ast_lines {
  _ast_result=$(jq -c --argjson start "$1" --argjson end "$2" '. + {lines: [$start, $end]}' <<< "$_ast_result")
}

function ast_words {
  local input=$1 word="" quote="" char i
  _ast_words=()
  for (( i=0; i<${#input}; i++ )); do
    char=${input:i:1}
    if [[ -n $quote ]]; then
      word+=$char
      [[ $char == "$quote" ]] && quote=""
    elif [[ $char == [\"\'] ]]; then
      word+=$char
      quote=$char
    elif [[ $char == [[:space:]] ]]; then
      [[ -n $word ]] && _ast_words+=("$word")
      word=""
    else
      word+=$char
    fi
  done
  [[ -n $word ]] && _ast_words+=("$word")
}

function ast_block {
  local parent_indent=$1 nodes=()
  while ast_peek && (( _ast_indent > parent_indent )); do
    ast_statement
    nodes+=("$_ast_result")
  done
  ast_list "${nodes[@]}"
}

function ast_statement {
  local start=$(( _ast_idx + 1 ))
  local if_re='^if[[:space:]]+(.+)$'
  local orphan_re='^(elif|else)([[:space:]]|$)'
  local for_re='^for[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]+in[[:space:]]+(.+)$'
  local while_re='^while[[:space:]]+(.+)$'
  local echo_re='^->[[:space:]]*(.*)$'
  local macro_re='^@([a-zA-Z_][a-zA-Z0-9_]*)([[:space:]]+(.*))?$'
  local function_re='^\.?[a-zA-Z_][a-zA-Z0-9_]*\('
  local declaration_re='^([a-zA-Z_][a-zA-Z0-9_]*)<([^>]*)>$'
  local typed_assignment_re='^([a-zA-Z_][a-zA-Z0-9_]*)<([^>]*)>[[:space:]]*=[[:space:]]*(.*)$'
  local assignment_re='^([a-zA-Z_][a-zA-Z0-9_]*)=(.*)$'

  if [[ $_ast_text =~ $if_re ]]; then
    ast_if "${BASH_REMATCH[1]}"
  elif [[ $_ast_text =~ $orphan_re ]]; then
    ast_error "'${BASH_REMATCH[1]}' without matching 'if'"
  elif [[ $_ast_text =~ $for_re ]]; then
    ast_for "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
  elif [[ $_ast_text =~ $while_re ]]; then
    ast_while "${BASH_REMATCH[1]}"
  elif [[ $_ast_text =~ $echo_re ]]; then
    ast_next
    ast_echo "${BASH_REMATCH[1]}"
  elif [[ $_ast_text =~ $macro_re ]]; then
    ast_next
    ast_macro "${BASH_REMATCH[1]}" "${BASH_REMATCH[3]}"
  elif [[ $_ast_text =~ $function_re ]]; then
    ast_function
  elif [[ $_ast_text =~ $declaration_re ]]; then
    ast_next
    ast_declaration "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
  elif [[ $_ast_text =~ $typed_assignment_re ]]; then
    ast_next
    ast_assignment "${BASH_REMATCH[1]}" "${BASH_REMATCH[3]}" "${BASH_REMATCH[2]}"
  elif [[ $_ast_text =~ $assignment_re ]]; then
    ast_next
    ast_assignment "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
  else
    ast_bash
  fi
  ast_lines "$start" "$_ast_last"
}

function ast_condition {
  local raw=$1 kind=test negated=false
  local negated_re='^![[:space:]]+(.*)$'
  local bool_re='^\$(\{[a-zA-Z_][a-zA-Z0-9_]*\}|--?[a-zA-Z_][a-zA-Z0-9_-]*|[a-zA-Z_][a-zA-Z0-9_]*)$'
  if [[ $raw =~ $negated_re ]]; then
    negated=true
    raw=${BASH_REMATCH[1]}
  fi
  if [[ $raw =~ $bool_re ]]; then
    kind=bool
  elif [[ $raw != [\$\"\'0-9-]* ]]; then
    kind=command
  fi
  _ast_result=$(jq -nc --arg kind "$kind" --arg raw "$raw" --argjson negated "$negated" '{kind: $kind, raw: $raw, negated: $negated}')
}

function ast_branch {
  local body=$_ast_result
  ast_condition "$1"
  _ast_result=$(jq -nc --argjson condition "$_ast_result" --argjson body "$body" '{condition: $condition, body: $body}')
}

function ast_if {
  local indent=$_ast_indent condition=$1 branches=() else_body=null
  local elif_re='^elif[[:space:]]+(.+)$'
  ast_next
  ast_block "$indent"
  ast_branch "$condition"
  branches+=("$_ast_result")
  while ast_peek && (( _ast_indent == indent )) && [[ $_ast_text =~ $elif_re ]]; do
    condition=${BASH_REMATCH[1]}
    ast_next
    ast_block "$indent"
    ast_branch "$condition"
    branches+=("$_ast_result")
  done
  if ast_peek && (( _ast_indent == indent )) && [[ $_ast_text == else ]]; then
    ast_next
    ast_block "$indent"
    else_body=$_ast_result
  fi
  ast_list "${branches[@]}"
  _ast_result=$(jq -c --argjson else_body "$else_body" '{type: "if", branches: ., else: $else_body}' <<< "$_ast_result")
}

function ast_for {
  local indent=$_ast_indent var=$1 items
  ast_next
  ast_words "$2"
  items=$(jq -nc '$ARGS.positional' --args -- "${_ast_words[@]}")
  ast_block "$indent"
  _ast_result=$(jq -nc --arg var "$var" --argjson items "$items" --argjson body "$_ast_result" '{type: "for", var: $var, items: $items, body: $body}')
}

function ast_while {
  local indent=$_ast_indent condition
  ast_condition "$1"
  condition=$_ast_result
  ast_next
  ast_block "$indent"
  _ast_result=$(jq -nc --argjson condition "$condition" --argjson body "$_ast_result" '{type: "while", condition: $condition, body: $body}')
}

function ast_echo {
  ast_value "$1"
  _ast_result=$(jq -c '{type: "echo", value: .}' <<< "$_ast_result")
}

function ast_macro {
  ast_words "$2"
  _ast_result=$(jq -nc --arg name "$1" '{type: "macro", name: $name, args: $ARGS.positional}' --args -- "${_ast_words[@]}")
}

function ast_bash {
  local indent=$_ast_indent lines=("$_ast_text")
  ast_next
  while ast_peek && (( _ast_indent > indent )); do
    lines+=("${_ast_lines[_ast_idx]:indent}")
    ast_next
  done
  _ast_result=$(jq -nc '{type: "bash", raw: ($ARGS.positional | join("\n"))}' --args -- "${lines[@]}")
}

function ast_value {
  local raw=$1
  local list_re='^\((.*)\)$'
  local math_re="^[^\"'()]+[[:space:]][-+*/%][[:space:]][^\"'()]+\$"
  if [[ $raw =~ $list_re ]]; then
    ast_words "${BASH_REMATCH[1]}"
    _ast_result=$(jq -nc '
      $ARGS.positional
      | if length > 0 and all(test("^[^=]+="))
        then {kind: "dict", entries: (map(capture("^(?<key>[^=]+)=(?<value>.*)$")) | from_entries)}
        else {kind: "list", items: .}
        end
    ' --args -- "${_ast_words[@]}")
  elif [[ $raw =~ $math_re ]]; then
    _ast_result=$(jq -nc --arg raw "$raw" '{kind: "math", raw: $raw}')
  else
    _ast_result=$(jq -nc --arg raw "$raw" '{kind: "scalar", raw: $raw}')
  fi
}

function ast_type {
  local type=$1 literals="" has_literals=false
  local literals_re='^([^({]*)[({](.*)[)}]$'
  if [[ $type =~ $literals_re ]]; then
    type=${BASH_REMATCH[1]}
    literals=${BASH_REMATCH[2]}
    has_literals=true
  fi
  ast_words "$literals"
  _ast_result=$(jq -nc --arg type "$type" --argjson has_literals "$has_literals" '{
    datatype: [$type | splits("\\s+") | select(. != "")],
    literals: (if $has_literals then $ARGS.positional else null end)
  }' --args -- "${_ast_words[@]}")
}

function ast_declaration {
  ast_type "$2"
  _ast_result=$(jq -c --arg name "$1" '{type: "declaration", name: $name} + .' <<< "$_ast_result")
}

function ast_assignment {
  local name=$1 value=$2 type=$3 datatype='{"datatype": null, "literals": null}'
  if [[ -n $type ]]; then
    ast_type "$type"
    datatype=$_ast_result
  fi
  ast_value "$value"
  _ast_result=$(jq -c --arg name "$name" --argjson datatype "$datatype" '{type: "assignment", name: $name} + $datatype + {value: .}' <<< "$_ast_result")
}

function ast_function {
  local indent=$_ast_indent start=$(( _ast_idx + 1 )) kind=function
  local name=${_ast_text%%\(*} text=${_ast_text#*\(}
  local params="" tail="" depth=1 char i param_list inline="" body
  local comment_re='(^|[[:space:]])#.*$'
  local echo_re='^->[[:space:]]*(.*)$'

  if [[ $name == .* ]]; then
    kind=command
    name=${name#.}
  fi

  while true; do
    if [[ $text =~ $comment_re ]]; then
      text=${text%"${BASH_REMATCH[0]}"}
    fi
    for (( i=0; i<${#text}; i++ )); do
      char=${text:i:1}
      [[ $char == "(" ]] && depth=$(( depth + 1 ))
      [[ $char == ")" ]] && depth=$(( depth - 1 ))
      if (( depth == 0 )); then
        tail=${text:i+1}
        break 2
      fi
      params+=$char
    done
    params+=" "
    _ast_idx=$(( _ast_idx + 1 ))
    (( _ast_idx < ${#_ast_lines[@]} )) || ast_error "unclosed '(' in $name"
    text=${_ast_lines[_ast_idx]}
  done

  tail=${tail#"${tail%%[![:space:]]*}"}
  if [[ -n $tail && ! $tail =~ $echo_re ]]; then
    ast_error "unexpected '$tail' after $name(...)"
  fi

  ast_params "$params"
  param_list=$_ast_result
  ast_next

  if [[ $tail =~ $echo_re ]]; then
    ast_echo "${BASH_REMATCH[1]}"
    ast_lines "$_ast_last" "$_ast_last"
    inline=$_ast_result
  fi

  ast_block "$indent"
  body=$_ast_result
  if [[ -n $inline ]]; then
    body=$(jq -c --argjson inline "$inline" '[$inline] + .' <<< "$body")
  fi

  _ast_result=$(jq -nc --arg type "$kind" --arg name "$name" --argjson params "$param_list" --argjson body "$body" '{type: $type, name: $name, params: $params, body: $body}')
}

function ast_param_tokens {
  local input=$1 word="" char i in_type=false
  _ast_tokens=()
  for (( i=0; i<${#input}; i++ )); do
    char=${input:i:1}
    if [[ $in_type == true ]]; then
      word+=$char
      [[ $char == ">" ]] && in_type=false
    elif [[ $char == "<" ]]; then
      if [[ -z $word ]] && (( ${#_ast_tokens[@]} > 0 )) && [[ ${_ast_tokens[-1]} != [\[\]\|] ]]; then
        word=${_ast_tokens[-1]}
        unset '_ast_tokens[-1]'
      fi
      word+=$char
      in_type=true
    elif [[ $char == [\[\]\|] ]]; then
      [[ -n $word ]] && _ast_tokens+=("$word")
      _ast_tokens+=("$char")
      word=""
    elif [[ $char == [[:space:]] ]]; then
      [[ -n $word ]] && _ast_tokens+=("$word")
      word=""
    else
      word+=$char
    fi
  done
  [[ -n $word ]] && _ast_tokens+=("$word")
}

function ast_params {
  ast_param_tokens "$1"
  _ast_pos=0
  ast_param_sequence
  if (( _ast_pos < ${#_ast_tokens[@]} )); then
    ast_error "unexpected '${_ast_tokens[_ast_pos]}' in parameters"
  fi
}

function ast_param_sequence {
  local items=()
  while (( _ast_pos < ${#_ast_tokens[@]} )) && [[ ${_ast_tokens[_ast_pos]} != "]" ]]; do
    ast_param_alternatives
    items+=("$_ast_result")
  done
  ast_list "${items[@]}"
}

function ast_param_alternatives {
  local options=()
  ast_param_item
  options+=("$_ast_result")
  while [[ ${_ast_tokens[_ast_pos]} == "|" ]]; do
    _ast_pos=$(( _ast_pos + 1 ))
    ast_param_item
    options+=("$_ast_result")
  done
  if (( ${#options[@]} > 1 )); then
    ast_list "${options[@]}"
    _ast_result=$(jq -c '
      def long: .flag | startswith("--");
      if length == 2 and all(.[]; .kind == "opt") and ([.[] | long] | sort) == [false, true]
      then (map(select(long)) | .[0]) + {aliases: [.[] | select(long | not) | .flag]}
      else {kind: "exclusive", options: .}
      end
    ' <<< "$_ast_result")
  fi
}

function ast_param_item {
  local token=${_ast_tokens[_ast_pos]} repeat=false
  _ast_pos=$(( _ast_pos + 1 ))
  if [[ $token != "[" ]]; then
    ast_param "$token"
    return
  fi
  ast_param_sequence
  [[ ${_ast_tokens[_ast_pos]} == "]" ]] || ast_error "missing ']' in parameters"
  _ast_pos=$(( _ast_pos + 1 ))
  if [[ ${_ast_tokens[_ast_pos]} == "..." ]]; then
    repeat=true
    _ast_pos=$(( _ast_pos + 1 ))
  fi
  _ast_result=$(jq -c --argjson repeat "$repeat" '
    if length == 1 and (.[0].kind == "arg" or .[0].kind == "opt" or .[0].kind == "rest")
    then .[0] + {optional: true, repeat: (.[0].repeat or $repeat)}
    else {kind: "group", optional: true, repeat: $repeat, params: .}
    end
  ' <<< "$_ast_result")
}

function ast_param {
  local token=$1
  local param_re='^(\*|-{0,2})([a-zA-Z_][a-zA-Z0-9_-]*)(<([^>]*)>)?(\.\.\.)?$'
  local default_re='^([^=]*)=(.*)$'
  [[ $token =~ $param_re ]] || ast_error "invalid parameter '$token'"
  local prefix=${BASH_REMATCH[1]} name=${BASH_REMATCH[2]} type=${BASH_REMATCH[4]} repeat=false
  local kind=arg flag="" default_type=str default="" has_default=false
  [[ -n ${BASH_REMATCH[5]} ]] && repeat=true
  if [[ $prefix == '*' ]]; then
    kind=rest
  elif [[ -n $prefix ]]; then
    kind=opt
    flag=$prefix$name
    default_type=bool
  fi
  if [[ $type =~ $default_re ]]; then
    type=${BASH_REMATCH[1]}
    default=${BASH_REMATCH[2]}
    has_default=true
  fi
  ast_type "${type:-$default_type}"
  _ast_result=$(jq -c \
    --arg kind "$kind" \
    --arg name "${name//-/_}" \
    --arg flag "$flag" \
    --argjson repeat "$repeat" \
    --arg default "$default" \
    --argjson has_default "$has_default" '
    {kind: $kind, name: $name, flag: (if $flag == "" then null else $flag end), optional: false, repeat: $repeat}
    + .
    + {default: (if $has_default then $default else null end)}
  ' <<< "$_ast_result")
}

if [[ ${0##*/} == ast.bash ]]; then
  ast "$@"
fi
