# Scriptopotamus Vision

## Overview
* Scriptopotamus is a psuedo-language that compiles to bash
* It is meant to clean up some of the nasty bash syntax, and also help enforce typing etc.
* It is written in pure bash

## Compiler
* The scriptopotamus compiler is loosely inspired by rust

### Syntax
#### Conditionals, For Loops, While Loops
```bash
if $condition
  echo "its true"
else
  echo "its false"

for i in $var
  echo $i

while $var
  echo "its true"

# COMPILES TO
for i in "${var[@]}"; do
  echo $i
done

while [[ $var == true ]]; do
  echo "its true"
done

if [[ $condition == true ]]; then
  echo "its true"
else
  echo "its false"
fi
```

#### Declaring Variables
* when declaring a variable with a type, scriptopotamus identifies all subsequent assignments to that variable and inserts a type check
```bash
my_num<int>="asd"
# compiles to
declare -i my_num="asd"
if [[ $my_num != "asd" ]]; then
  echo "my_num: expected int, recieved str"
fi
```
* lists and dicts are declared with `<list type>` and `<dict type>`
```bash
my_list<list int>=(1 2 3)
my_dict<dict int str>=(a=1 b=2 c=3)

a=$my_dict[a]
i0=$my_list[0]

# COMPILES TO
declare -a my_list=(1 2 3)
declare -A my_dict=(
  [a]=1
  [b]=2
  [c]=3
)
a=${my_dict[a]}
i0=${my_list[0]}
```
* types are `str (literals...)`, `int (literals...)`, `float (literals...)`, `bool`, `list <type>`, `dict <type> <type>`, `const <type>`, `literal <type> {<value>...}`
```bash 
my_const<const str>=hello
my_literal<str (a b c)>
my_literal<int (1 2 3)>
```

#### Handling Arithmetic
```bash
PI<const float>=3.14
result=$PI * $diameter

# COMPILES TO
declare -r PI=3.14
result=$(bc <<< "$PI * $diameter")
```

#### Defining Functions
* functions must have nested bodies, but can be inlined
* `->` is an alias for echo
```bash
circumference(diameter<float>)
  -> $PI * $diameter

circumference(diameter<float>) -> $PI * $diameter

# BOTH COMPILE TO
function circumference {
  echo $(bc <<< "$PI * $diameter")
}
```
* `[arg]` or `[--opt]` are optional args or opts
* `|` delineates mutual exclusivity
* `...` means repeatable
* `[--opt [--opt2]]` means optional opt with optional opt2 (where opt2 is only valid if opt is used)
* args and opts can have types, syntax mirrors variable declaration
  * default arg type is `str`
  * default opt type is `bool`
* defaults declared with `=`

```bash
my_func(
  paths...
  [--default-flag<str="default">]
  [--optional_flag<str {a b c}>]
  [--options<str>... [--multi]] | [--confirm]
```
* scriptopotamus automatically parses the args and opts
```bash
my_func(
    path
    [optional_arg<int>]
    -v<str>
    [
        -s | --select
        --options<str>...
        [--multi]
    ] | [
        --confirm # no type on opts = bool
    ]
)
    if $--select
        for i in $options
            echo $i
    else
        echo "nope"

# COMPILES TO
function my_func {
  local _arg_path= _opt_v= _opt_select= _opt_options= _opt_multi= _opt_confirm=
  local declare -i _arg_optional_arg=
  # var assignment
  while (( $# > 0 )); do
		case "$1" in
			-v) _opt_v=1; shift ;;
			*)  path="$1"; shift ;;
		esac
	done
  # input check
  if [[ -z $_arg_path ]]; then
    echo "<path> argument required"
    exit 1
  fi
  if [[ -n $_arg_optional_arg && $() ]]; then
    echo "<path> argument required"
    exit 1
  fi
  if [[ -z "$_v" ]]; then
    echo "-v opt required"
    exit 1
  fi
  if [[ -n $__select && -n $__confirm ]];
    echo "--select and --confirm mutually exclusive"
    exit 1
  fi
  # ... and so on and so forth

  # user code
  if [[ $_opt_select == true ]]; then
    for i in "${_opt_options[@]}"; do
        echo $i
    done
  else
    echo "nope"
  fi
}
```

#### Defining Commands
* scriptopotamus allows for command definition for scripts
* defined by putting a . in front of a function
```bash
# script name: "test"
.my_cmd(arg1 [--opt1<str>])
  .my_subcmd(arg2)
    -> echo "$arg2"
  -> echo "$arg1" "$--opt1"
```
```bash
./test my_cmd "hello" --opt1 "world" # -> "hello world"
./test my_cmd my_subcmd "world" # -> "world"
```

#### Builtins
* scriptopotamus provides some builtins for common tasks
```bash
# throw: exits with red error message
throw(code<int> msg)

# prt: echo with style
prt(
    msg...
    [-s|--style <str>...]...
    [-n]
)

# wrapper around read
input(
  prompt 
  -v<str>
  [
      --select 
      --options<str>...
      [--multi]
  ] | [ # | delineates mutual exclusivity
      --confirm
  ]
)
```
```bash
# examples
throw 1 "bad input"
prt -s "red" "hello" -n
input "prompt" -v "var" --select --options "option1" "option2" --multi
input "prompt" -v "var" --confirm
```

### Implementation
