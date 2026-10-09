#!/usr/bin/env bash

declare -gA _compile_types=()
declare -gA _compile_imported=()
declare -ga _compile_commands=()
declare -gA _compile_helpers=()
declare -ga _compile_prelude=()
declare -g _compile_root=""
declare -gA _compile_patterns=(
  [int]='^-?[0-9]+$'
  [float]='^-?[0-9]*\.?[0-9]+$'
  [bool]='^(true|false)$'
)

_compile_var_re='^\$\{?([a-zA-Z_][a-zA-Z0-9_]*)\}?$'

_compile_type_fields='[
  (.datatype | map(select(. != "const")) | .[0] // ""),
  (.datatype | index("const") != null),
  (.literals // [] | join("|")),
  ((.datatype | join(" ")) + (if .literals then " (\(.literals | join(" ")))" else "" end))
]'

_compile_param_jq='
def annotate($repeat):
  map(
    if .kind == "group" then .repeat as $r | .params |= annotate($repeat or $r)
    elif .kind == "exclusive" then .options |= annotate($repeat)
    else . + {array: (.kind == "rest" or ((.repeat or $repeat) and .datatype != ["bool"]))}
    end
  );

def leaves:
  .[]
  | if .kind == "group" then .params | leaves
    elif .kind == "exclusive" then .options | leaves
    else .
    end;

def used:
  if .kind == "group" then "( " + ([.params[] | used] | join(" || ")) + " )"
  elif .kind == "exclusive" then "( " + ([.options[] | used] | join(" || ")) + " )"
  elif .array then "${#\(.name)[@]} -gt 0"
  elif .kind == "opt" and .datatype == ["bool"] then "$\(.name) == true"
  else "-n $\(.name)"
  end;

def name_of:
  if .kind == "group" then "[" + ([.params[] | name_of] | join(" ")) + "]"
  elif .kind == "exclusive" then [.options[] | name_of] | join(" | ")
  elif .kind == "rest" then "*\(.name)"
  elif .kind == "arg" then "<\(.name)>"
  else .flag
  end;

def when($context; $test):
  if $context == null then $test else "\($context) && \($test)" end;

def checks($context):
  .[]
  | if .kind == "group" then used as $used | .params | checks($used)
    elif .kind == "exclusive" then
      .options as $options
      | (
          range(0; $options | length) as $i
          | range($i + 1; $options | length) as $j
          | {
              test: when($context; "\($options[$i] | used) && \($options[$j] | used)"),
              message: "\($options[$i] | name_of) and \($options[$j] | name_of) are mutually exclusive"
            }
        ),
        (
          if any($options[]; .optional) then empty
          else {test: when($context; "! \(used)"), message: "one of \(name_of) is required"}
          end
        ),
        ($options[] | select(.kind == "group") | used as $used | .params | checks($used))
    elif .optional then empty
    else {test: when($context; "! ( \(used) )"), message: "\(name_of) is required"}
    end;
'

function compile {
  local path=${1:?"compile: <path> Required"} tree nodes node
  tree=$(ast "$path") || exit 1
  _compile_output=()
  _compile_depth=0
  _compile_types=()
  _compile_imported=()
  _compile_commands=()
  _compile_helpers=()
  _compile_prelude=()
  _compile_root=""
  _compile_import_dir=${SCRIPPO_IMPORT_DIR:-$(dirname "$path")}
  macro_import_remember "$path"
  [[ -z ${SCRIPPO_SOURCE:-} ]] || macro_import_remember "$SCRIPPO_SOURCE"
  compile_emit "#!/usr/bin/env bash"
  mapfile -t nodes < <(jq -c '.body[]' <<< "$tree")
  for node in "${nodes[@]}"; do
    compile_node "$node"
  done
  compile_dispatch
  printf '%s\n' "${_compile_output[0]}" "${_compile_prelude[@]}" "${_compile_output[@]:1}"
}

function compile_error {
  echo "compile: line $(jq -r '.lines[0]' <<< "$1"): $2" >&2
  exit 1
}

function compile_read {
  IFS=$'\x1f' read -r "${@:3}" < <(jq -r "$2 | map(. // \"\" | tostring) | join(\"\\u001f\")" <<< "$1")
}

function compile_emit {
  local pad line
  printf -v pad '%*s' $(( _compile_depth * 2 )) ''
  while IFS= read -r line; do
    _compile_output+=("$pad$line")
  done <<< "$1"
}

function compile_helper {
  local name=$1 line
  [[ -z ${_compile_helpers[$name]} ]] || return 0
  _compile_helpers[$name]=1
  while IFS= read -r line; do
    _compile_prelude+=("$line")
  done
  _compile_prelude+=("")
}

function compile_indent {
  _compile_depth=$(( _compile_depth + 1 ))
}

function compile_dedent {
  _compile_depth=$(( _compile_depth - 1 ))
}

function compile_fail {
  compile_emit "if $1; then"
  compile_emit "  echo \"$2\" >&2"
  compile_emit "  exit 1"
  compile_emit "fi"
}

function compile_node {
  local node=$1 type
  type=$(jq -r .type <<< "$node")
  case $type in
    if) compile_if "$node" ;;
    for) compile_for "$node" ;;
    while) compile_while "$node" ;;
    case) compile_case "$node" ;;
    echo) compile_echo "$node" ;;
    declaration) compile_declaration "$node" ;;
    assignment) compile_assignment "$node" ;;
    function|command) compile_function "$node" ;;
    bash) compile_bash "$node" ;;
    macro) compile_macro "$node" ;;
  esac
}

function compile_body {
  local nodes node
  mapfile -t nodes < <(jq -c '.body[] | select(.type != "command")' <<< "$1")
  compile_indent
  for node in "${nodes[@]}"; do
    compile_node "$node"
  done
  if (( ${#nodes[@]} == 0 )); then
    compile_emit ":"
  fi
  compile_dedent
}

function compile_expr {
  local text=$1 replacement
  local opt_re='\$--?([a-zA-Z_][a-zA-Z0-9_-]*)'
  local index_re='\$([a-zA-Z_][a-zA-Z0-9_]*)\[([^]]*)\]'
  local operand='(\$\{?[a-zA-Z_][a-zA-Z0-9_]*(\[[^]]*\])?\}?|[0-9]*\.?[0-9]+)'
  local math_re='\$\(([[:space:]]*'"$operand"'([[:space:]]*[-+*/%^][[:space:]]*'"$operand"')+[[:space:]]*)\)'

  macro_rootdir_expand "$text"
  text=$_compile_result
  while [[ $text =~ $opt_re ]]; do
    replacement="\$${BASH_REMATCH[1]//-/_}"
    text=${text/"${BASH_REMATCH[0]}"/"$replacement"}
  done
  while [[ $text =~ $index_re ]]; do
    replacement="\${${BASH_REMATCH[1]}[${BASH_REMATCH[2]}]}"
    text=${text/"${BASH_REMATCH[0]}"/"$replacement"}
  done
  while [[ $text =~ $math_re ]]; do
    replacement="\$(bc -l <<< \"${BASH_REMATCH[1]}\")"
    text=${text/"${BASH_REMATCH[0]}"/"$replacement"}
  done
  _compile_result=$text
}

function compile_condition {
  local kind raw negated
  compile_read "$1" '[.kind, .raw, .negated]' kind raw negated
  compile_expr "$raw"
  if [[ $kind == bool ]]; then
    _compile_result="[[ $_compile_result == true ]]"
  elif [[ $kind == test ]]; then
    _compile_result="[[ $_compile_result ]]"
  fi
  if [[ $negated == true ]]; then
    _compile_result="! $_compile_result"
  fi
}

function compile_value {
  local value=$1 base=$2 kind raw
  compile_read "$value" '[.kind, .raw]' kind raw
  if [[ $kind == list ]]; then
    raw=$(jq -r '"(" + (.items | join(" ")) + ")"' <<< "$value")
  elif [[ $kind == dict ]]; then
    raw=$(jq -r '"(\n" + (.entries | to_entries | map("  [\(.key)]=\(.value)") | join("\n")) + "\n)"' <<< "$value")
  fi
  compile_expr "$raw"
  if [[ $kind == math && $base == int ]]; then
    _compile_result="\$(( $_compile_result ))"
  elif [[ $kind == math ]]; then
    _compile_result="\$(bc -l <<< \"$_compile_result\")"
  fi
}

function compile_flags {
  local base=$1 const=$2 flags=""
  case $base in
    int) flags=i ;;
    list) flags=a ;;
    dict) flags=A ;;
    upper) flags=u ;;
    lower) flags=l ;;
  esac
  if [[ $const == true ]]; then
    flags+=r
  fi
  _compile_result=${flags:+ -$flags}
}

function compile_type_test {
  local var=$1 base=$2 literals=$3
  _compile_result=""
  if [[ -n $literals ]]; then
    _compile_result="$var != @($literals)"
  elif [[ -n ${_compile_patterns[$base]} ]]; then
    _compile_result="! $var =~ ${_compile_patterns[$base]}"
  fi
}

function compile_literal {
  local raw=$1
  local double_re='^"([^"$`\]*)"$'
  local single_re="^'([^']*)'\$"
  local bare_re='^[a-zA-Z0-9_.+-]+$'
  if [[ $raw =~ $double_re || $raw =~ $single_re ]]; then
    _compile_result=${BASH_REMATCH[1]}
    return 0
  fi
  if [[ $raw =~ $bare_re ]]; then
    _compile_result=$raw
    return 0
  fi
  return 1
}

function compile_accepts {
  local value=$1 base=$2 literals=$3
  if [[ -n $literals ]]; then
    [[ "|$literals|" == *"|$value|"* ]]
    return
  fi
  [[ -z ${_compile_patterns[$base]} || $value =~ ${_compile_patterns[$base]} ]]
}

function compile_if {
  local node=$1 branches branch keyword=if
  mapfile -t branches < <(jq -c '.branches[]' <<< "$node")
  for branch in "${branches[@]}"; do
    compile_condition "$(jq -c .condition <<< "$branch")"
    compile_emit "$keyword $_compile_result; then"
    compile_body "$branch"
    keyword=elif
  done
  if [[ $(jq '.else != null' <<< "$node") == true ]]; then
    compile_emit "else"
    compile_body "$(jq -c '{body: .else}' <<< "$node")"
  fi
  compile_emit "fi"
}

function compile_for {
  local node=$1 var raw_items item items=()
  var=$(jq -r .var <<< "$node")
  mapfile -t raw_items < <(jq -r '.items[]' <<< "$node")
  for item in "${raw_items[@]}"; do
    compile_expr "$item"
    if [[ $_compile_result =~ $_compile_var_re ]]; then
      items+=("\"\${${BASH_REMATCH[1]}[@]}\"")
    else
      items+=("$_compile_result")
    fi
  done
  compile_emit "for $var in ${items[*]}; do"
  compile_body "$node"
  compile_emit "done"
}

function compile_while {
  local node=$1
  compile_condition "$(jq -c .condition <<< "$node")"
  compile_emit "while $_compile_result; do"
  compile_body "$node"
  compile_emit "done"
}

function compile_case {
  local node=$1 arms arm pattern terminator
  compile_expr "$(jq -r .subject <<< "$node")"
  compile_emit "case $_compile_result in"
  compile_indent
  mapfile -t arms < <(jq -c '.arms[]' <<< "$node")
  for arm in "${arms[@]}"; do
    compile_read "$arm" '[.pattern, .terminator]' pattern terminator
    compile_expr "$pattern"
    compile_emit "$_compile_result)"
    compile_body "$arm"
    compile_emit "  $terminator"
  done
  compile_dedent
  compile_emit "esac"
}

function compile_echo {
  compile_value "$(jq -c .value <<< "$1")" ""
  compile_emit "echo $_compile_result"
}

function compile_bash {
  local node=$1 raw
  local closer_re='^(fi|done|then|do|esac)([[:space:];]|$)'
  raw=$(jq -r .raw <<< "$node")
  if [[ $raw =~ $closer_re ]]; then
    compile_error "$node" "unexpected '${BASH_REMATCH[1]}': blocks are closed by indentation"
  fi
  compile_expr "$raw"
  compile_emit "$_compile_result"
}

function compile_declaration {
  local node=$1 name type_json base const literals description
  name=$(jq -r .name <<< "$node")
  type_json=$(jq -c '{datatype, literals}' <<< "$node")
  _compile_types[$name]=$type_json
  compile_read "$type_json" "$_compile_type_fields" base const literals description
  compile_flags "$base" "$const"
  compile_emit "declare$_compile_result $name"
}

function compile_assignment {
  local node=$1 name typed kind raw value type_json
  local base const literals description prefix="" code test
  compile_read "$node" '[.name, (.datatype != null), .value.kind, .value.raw]' name typed kind raw
  value=$(jq -c .value <<< "$node")

  if [[ $typed == true ]]; then
    type_json=$(jq -c '{datatype, literals}' <<< "$node")
    _compile_types[$name]=$type_json
  else
    type_json=${_compile_types[$name]}
  fi

  if [[ -z $type_json ]]; then
    compile_value "$value" ""
    compile_emit "$name=$_compile_result"
    return
  fi

  compile_read "$type_json" "$_compile_type_fields" base const literals description
  if [[ $typed == true ]]; then
    compile_flags "$base" "$const"
    prefix="declare$_compile_result "
  elif [[ $const == true ]]; then
    compile_error "$node" "cannot reassign const '$name'"
  fi

  compile_value "$value" "$base"
  code=$_compile_result
  compile_type_test '$_scrippo_value' "$base" "$literals"
  test=$_compile_result

  if [[ -z $test || $kind == list || $kind == dict ]]; then
    compile_emit "$prefix$name=$code"
    return
  fi

  if [[ $kind == scalar ]] && compile_literal "$raw"; then
    if ! compile_accepts "$_compile_result" "$base" "$literals"; then
      compile_error "$node" "$name: expected $description, received $raw"
    fi
    compile_emit "$prefix$name=$code"
    return
  fi

  compile_emit "_scrippo_value=$code"
  compile_fail "[[ -n \$_scrippo_value && $test ]]" "$name: expected $description, received '\$_scrippo_value'"
  compile_emit "$prefix$name=\$_scrippo_value"
}

function compile_function {
  local node=$1 name=$2 type subcommands sub subname label
  if [[ -z $name ]]; then
    compile_read "$node" '[.name, .type]' name type
    if [[ $type == command ]] && (( _compile_depth == 0 )); then
      compile_command "$node" "$name"
    fi
  fi
  label=${name//__/ }
  if [[ -z $name ]]; then
    name=_scrippo_root
    label='${0##*/}'
  fi
  mapfile -t subcommands < <(jq -c '.body[] | select(.type == "command")' <<< "$node")

  for sub in "${subcommands[@]}"; do
    subname=$(jq -r .name <<< "$sub")
    if [[ $name == _scrippo_root ]]; then
      compile_error "$sub" "commands cannot be nested inside the root command '.()'"
    elif [[ -z $subname ]]; then
      compile_error "$sub" "the root command '.()' must be defined at the top level"
    fi
    compile_function "$sub" "${name}__$subname"
  done

  compile_emit "function $name {"
  compile_indent
  for sub in "${subcommands[@]}"; do
    subname=$(jq -r .name <<< "$sub")
    compile_emit "if [[ \$1 == $subname ]]; then"
    compile_emit "  shift"
    compile_emit "  ${name}__$subname \"\$@\""
    compile_emit "  return"
    compile_emit "fi"
  done
  compile_params "$node" "$label"
  compile_dedent
  compile_body "$node"
  compile_emit "}"
  if (( _compile_depth == 0 )); then
    compile_emit ""
  fi
}

function compile_params {
  local node=$1 fn=$2 leaves leaf checks check test message
  local kind name flag array multi default base const literals description pattern
  local position=0 rest=false catchall="" catchall_at=-1 index=0

  mapfile -t leaves < <(jq -c "$_compile_param_jq"'.params | annotate(false) | leaves' <<< "$node")
  if (( ${#leaves[@]} == 0 )); then
    return 0
  fi

  for leaf in "${leaves[@]}"; do
    compile_read "$leaf" '[.kind, .name]' kind name
    if [[ $kind == rest ]]; then
      if [[ -n $catchall ]]; then
        compile_error "$node" "only one *param is allowed"
      fi
      catchall=$name
      catchall_at=$index
    fi
    index=$(( index + 1 ))
  done
  if [[ -n $catchall && $catchall_at -ne $(( ${#leaves[@]} - 1 )) ]]; then
    compile_error "$node" "*$catchall must be the last parameter"
  fi

  for leaf in "${leaves[@]}"; do
    compile_read "$leaf" '[.kind, .name, .array, .default, .datatype[0]]' kind name array default base
    if [[ $kind == rest ]]; then
      compile_emit "local -a $name=()"
    elif [[ $kind != opt ]]; then
      continue
    elif [[ $array == true ]]; then
      compile_emit "local -a $name=()"
    elif [[ $base == bool ]]; then
      compile_emit "local $name=${default:-false}"
    else
      compile_emit "local $name${default:+=$default}"
    fi
  done

  if [[ -n $catchall ]]; then
    compile_emit "local -a _scrippo_seq=()"
    compile_emit "local -a _scrippo_kind=()"
  fi
  compile_emit "local _scrippo_args=()"
  compile_emit "while (( \$# > 0 )); do"
  compile_indent
  compile_emit "case \$1 in"
  compile_indent
  for leaf in "${leaves[@]}"; do
    compile_read "$leaf" '[.kind, .name, .flag, .array, .repeat, .datatype[0], ((.aliases // []) + [.flag] | join("|"))]' kind name flag array multi base pattern
    if [[ $kind == opt ]]; then
      compile_option "$fn" "$name" "$flag" "$array" "$multi" "$base" "$pattern"
    fi
  done
  if [[ -n $catchall ]]; then
    compile_emit "--)"
    compile_emit "  shift"
    compile_emit "  while (( \$# > 0 )); do"
    compile_emit "    _scrippo_seq+=(\"\$1\")"
    compile_emit "    _scrippo_kind+=(arg)"
    compile_emit "    shift"
    compile_emit "  done"
    compile_emit "  ;;"
    compile_emit "-?*)"
    compile_emit "  _scrippo_seq+=(\"\$1\")"
    compile_emit "  _scrippo_kind+=(flag)"
    compile_emit "  shift"
    compile_emit "  ;;"
    compile_emit "*)"
    compile_emit "  _scrippo_seq+=(\"\$1\")"
    compile_emit "  _scrippo_kind+=(arg)"
    compile_emit "  shift"
    compile_emit "  ;;"
  else
    compile_emit "--)"
    compile_emit "  shift"
    compile_emit "  _scrippo_args+=(\"\$@\")"
    compile_emit "  break"
    compile_emit "  ;;"
    compile_emit "-?*)"
    compile_emit "  echo \"$fn: unknown option '\$1'\" >&2"
    compile_emit "  exit 1"
    compile_emit "  ;;"
    compile_emit "*)"
    compile_emit "  _scrippo_args+=(\"\$1\")"
    compile_emit "  shift"
    compile_emit "  ;;"
  fi
  compile_dedent
  compile_emit "esac"
  compile_dedent
  compile_emit "done"

  if [[ -n $catchall ]]; then
    compile_emit "local _scrippo_i"
    compile_emit "for _scrippo_i in \"\${!_scrippo_seq[@]}\"; do"
    compile_emit "  if [[ \${_scrippo_kind[_scrippo_i]} == arg ]]; then"
    compile_emit "    _scrippo_args+=(\"\${_scrippo_seq[_scrippo_i]}\")"
    compile_emit "  fi"
    compile_emit "done"
  fi

  for leaf in "${leaves[@]}"; do
    compile_read "$leaf" '[.kind, .name, .array, .default]' kind name array default
    if [[ $kind != arg ]]; then
      continue
    elif [[ $array == true ]]; then
      compile_emit "local -a $name=(\"\${_scrippo_args[@]:$position}\")"
      rest=true
    else
      compile_emit "local $name=\${_scrippo_args[$position]${default:+:-$default}}"
      position=$(( position + 1 ))
    fi
  done
  if [[ -n $catchall ]]; then
    compile_rest "$catchall" "$position" "$rest"
  elif [[ $rest == false ]]; then
    compile_fail "(( \${#_scrippo_args[@]} > $position ))" "$fn: too many arguments"
  fi

  mapfile -t checks < <(jq -c "$_compile_param_jq"'.params | annotate(false) | checks(null)' <<< "$node")
  for check in "${checks[@]}"; do
    compile_read "$check" '[.test, .message]' test message
    compile_fail "[[ $test ]]" "$fn: $message"
  done

  for leaf in "${leaves[@]}"; do
    compile_read "$leaf" '[.kind, .name, .flag, .array]' kind name flag array
    compile_read "$leaf" "$_compile_type_fields" base const literals description
    if [[ $kind == opt && $base == bool ]]; then
      continue
    fi
    if [[ $kind == rest ]]; then
      flag="*$name"
    elif [[ $kind == arg ]]; then
      flag="<$name>"
    fi
    if [[ $array == true ]]; then
      compile_type_test '$_scrippo_item' "$base" "$literals"
      [[ -n $_compile_result ]] || continue
      compile_emit "for _scrippo_item in \"\${$name[@]}\"; do"
      compile_indent
      compile_fail "[[ $_compile_result ]]" "$fn: $flag expected $description, received '\$_scrippo_item'"
      compile_dedent
      compile_emit "done"
    else
      _compile_types[$name]=$(jq -c '{datatype, literals}' <<< "$leaf")
      compile_type_test "\$$name" "$base" "$literals"
      [[ -n $_compile_result ]] || continue
      compile_fail "[[ -n \$$name && $_compile_result ]]" "$fn: $flag expected $description, received '\$$name'"
    fi
  done
}

function compile_rest {
  local name=$1 skip=$2 takes_all=$3
  if [[ $takes_all == true ]]; then
    compile_emit "for _scrippo_i in \"\${!_scrippo_seq[@]}\"; do"
    compile_emit "  if [[ \${_scrippo_kind[_scrippo_i]} == flag ]]; then"
    compile_emit "    $name+=(\"\${_scrippo_seq[_scrippo_i]}\")"
    compile_emit "  fi"
    compile_emit "done"
  elif (( skip == 0 )); then
    compile_emit "$name=(\"\${_scrippo_seq[@]}\")"
  else
    compile_emit "local _scrippo_seen=0"
    compile_emit "for _scrippo_i in \"\${!_scrippo_seq[@]}\"; do"
    compile_emit "  if [[ \${_scrippo_kind[_scrippo_i]} == arg ]] && (( _scrippo_seen < $skip )); then"
    compile_emit "    _scrippo_seen=\$(( _scrippo_seen + 1 ))"
    compile_emit "    continue"
    compile_emit "  fi"
    compile_emit "  $name+=(\"\${_scrippo_seq[_scrippo_i]}\")"
    compile_emit "done"
  fi
}

function compile_option {
  local fn=$1 name=$2 flag=$3 array=$4 multi=$5 base=$6 pattern=$7
  compile_emit "$pattern)"
  compile_indent
  if [[ $base == bool && $array != true ]]; then
    compile_emit "$name=true"
    compile_emit "shift"
  elif [[ $multi == true ]]; then
    compile_emit "shift"
    compile_emit "while (( \$# > 0 )) && [[ \$1 != -?* ]]; do"
    compile_emit "  $name+=(\"\$1\")"
    compile_emit "  shift"
    compile_emit "done"
  else
    compile_fail "(( \$# < 2 ))" "$fn: $flag requires a value"
    if [[ $array == true ]]; then
      compile_emit "$name+=(\"\$2\")"
    else
      compile_emit "$name=\$2"
    fi
    compile_emit "shift 2"
  fi
  compile_emit ";;"
  compile_dedent
}

function compile_command {
  local node=$1 name=$2 params
  if [[ -z $name && -n $_compile_root ]]; then
    compile_error "$node" "root command '.()' already defined"
  elif [[ -z $name ]]; then
    params=$(jq "$_compile_param_jq"'[.params | leaves] | length' <<< "$node")
    _compile_root=bare
    (( params == 0 )) || _compile_root=params
    return 0
  fi
  if [[ " ${_compile_commands[*]} " == *" $name "* ]]; then
    compile_error "$node" "command '$name' already defined"
  fi
  _compile_commands+=("$name")
}

function compile_dispatch {
  local command
  if [[ -z $_compile_root ]] && (( ${#_compile_commands[@]} == 0 )); then
    return 0
  fi
  compile_emit "case \$1 in"
  compile_indent
  if [[ -z $_compile_root ]]; then
    compile_emit "\"\")"
    compile_emit "  echo \"\${0##*/}: command required\" >&2"
    compile_emit "  echo \"commands: ${_compile_commands[*]}\" >&2"
    compile_emit "  exit 1"
    compile_emit "  ;;"
  elif [[ $_compile_root == bare ]]; then
    compile_emit "\"\")"
    compile_emit "  _scrippo_root"
    compile_emit "  ;;"
  fi
  for command in "${_compile_commands[@]}"; do
    compile_emit "$command)"
    compile_emit "  shift"
    compile_emit "  $command \"\$@\""
    compile_emit "  ;;"
  done
  compile_emit "*)"
  if [[ $_compile_root == params ]]; then
    compile_emit "  _scrippo_root \"\$@\""
  else
    compile_emit "  echo \"\${0##*/}: unknown command '\$1'\" >&2"
    if (( ${#_compile_commands[@]} > 0 )); then
      compile_emit "  echo \"commands: ${_compile_commands[*]}\" >&2"
    fi
    compile_emit "  exit 1"
  fi
  compile_emit "  ;;"
  compile_dedent
  compile_emit "esac"
}

if [[ ${0##*/} == compiler.bash ]]; then
  _compile_dir=$(dirname "${BASH_SOURCE[0]}")
  source "$_compile_dir/ast.bash"
  source "$_compile_dir/macros.bash"
  for _compile_macro_file in "$_compile_dir"/macros/*.bash; do
    source "$_compile_macro_file"
  done
  compile "$@"
fi
