# Scriptopotamus Vision

## Overview
* Scriptopotamus is a psuedo-language that compiles to bash
* It is meant to clean up some of the nasty bash syntax, and also help enforce typing etc.

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
* they can also be short, one liners
```bash
if $condition; echo "its true"; else echo "its false"
while $condition; echo "its true"
for i in $var; echo $i
```

#### Case
* arms are `pattern)` followed by an inline statement, an indented body, or both
* `in` and `;;` are optional, `;&` and `;;&` can end an arm for fallthrough, and `esac` is an error like `fi`
```bash
case $str
  hello | hi) -> "Hi"
  world)
    -> "Bye"
  *) -> "?" ;;

# COMPILES TO
case $str in
  hello | hi)
    echo "Hi"
    ;;
  world)
    echo "Bye"
    ;;
  *)
    echo "?"
    ;;
esac
```

#### Declaring Variables
```bash
my_num<int>="asd"
# compiles to
declare -i my_num="asd"
if [[ $my_num != "asd" ]]; then
  echo "my_num: expected int, recieved str"
fi
```
* lists and dicts are declared with `<list>` and `<dict>`
```bash
my_list<list>=(1 2 3)
my_dict<dict>=(a=1 b=2 c=3)

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
* types are 
    - `str {literals...}`: a string value
    - `int {literals...}`: a integer value
    - `float {literals...}`: a float value
    - `bool`: a boolean value
    - `list`: a list of values of type <type>
    - `dict`: a dictionary with keys of type <type> and values of type <type>
    - `const`: a constant type that cannot be reassigned
    - `any`: any is a wildcard type that allows for any value
```bash 
my_const<const>=hello
my_literal_str<str {a b c}>
my_literal_int<int {1 2 3}>
my_float<float>
my_bool<bool>
my_list<list>=(1 2 3)
my_dict<dict>=(a=1 b=2 c=3)
my_any<any>
my_dict_any_any<dict>=(a=1 2=a c=3)
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

#### Macros
##### Builtin Macros
* macros are referenced with `@` and, like rust macros, are expanded inline at compile time
* there is no runtime library: each macro call is replaced by the bash it generates
* today, there is only a one builtin macro:
    * `@import`: imports a bash or scrippo file, or a library
        * a glob such as `@import ./*` imports every matching file in sorted order; directories are skipped
        * each file is only imported once, so repeated, circular, and self imports are skipped
        * top-level commands in an imported `.scrippo` file are added to the script's commands, and defining the same command twice is a compile error
```bash
@import lib/utils.bash # these always resolve relative to the file, not the current working directory
@import scrippo/*
@import scrippo/io
@import script.scrippo
```
##### Defining Macros
```bash
@my_macro()
	-> "hello"

risky_fn || {
    @my_macro
    prt -s green "World"
}
```

#### Libraries
##### Standard Library
* imported with `@import scrippo/std`
###### `scrippo/std/io`
* imported with `@import scrippo/std/io`
```bash
# throw: exits with red error message
throw(code<int> msg...)

# prt: echo with style
# styles: bold dim italic underline black red green yellow blue magenta cyan white
# a style applies only to the msg that follows it
# msgs are joined by a space unless -j is given
prt(
    msg...
    [-s|--style <str>...]...
    [-j|--join <str>]
    [-n]
)

# wrapper around read
input(
  prompt
  -v<str>
  [
      --select<str>...
      [--multi]
  ] | [ # | delineates mutual exclusivity
      --confirm
  ]
)

# example usage
throw 1 "bad input"
prt -s red bold "hello" -n
input "Enter a message" -v msg
input "Are you sure?" -v sure --confirm
input "Pick one" -v choice --select "a" "b"
```
* `input` emits its runtime helpers (`_scrippo_input`, `_scrippo_confirm`, `_scrippo_select`) once, at the top of the compiled script
* prompts are styled and drawn on stderr; once answered, the prompt collapses to a single `✔ prompt · answer` line
* `--confirm` answers on a single `y`/`n` keypress, and enter means no
* `--select` is an arrow-key menu (`↑↓` or `j`/`k`, enter to pick)
* `--select --multi` is a checkbox menu: space toggles, `a` toggles all, enter confirms, and the picks are stored as a list
* when stdin or stderr is not a terminal, `--select` falls back to a numbered list and reads space or comma separated numbers, `--confirm` reads a line starting with `y`, and text input is a plain `read`
###### `scrippo/std/types`
* imported with `@import scrippo/std/types`
```bash
list(
    array<list>
    [--join<str>]
    [--length]
)


dict(
    dict<dict>
    [--keys]
    [--values]
    [--length]
)

str(
    string<str>
    [--length]
    [--split<str>]
)

# usage
str "hello" --length # -> 5

my_ls<list>=("a" "b" "c")
list --join $my_ls "." # -> "a.b.c"
list --length $my_ls # -> 3

my_dict<dict>=(a=1 b=2 c=3)
dict --keys $my_dict # -> "a b c"
dict --values $my_dict # -> "1 2 3"
dict --length $my_dict # -> 3
```