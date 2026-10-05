# 🦛 scriptopotamus

* Inspired by bash, python, and sass
* Written entirely in bash

- Types
    - Primitive Types:
        - int
        - float
        - str / upper / lower
        - bool
    - Compound Types:
        - const T
        - list T
        - dict K T
    
    ```bash
    my_num<int>=1
    my_str<str>="hello"
    my_bool<bool>=false
    pi<const float>=3.14
    ```
    
    - type declarations are not necessary on var declarations, can be inferred
    - types are enforced at compile time
- Lists
    - lists are just easier bash indexed arrays
    - `len` is a builtin to get the length
    
    ```bash
    my_list<list int>=(1 2 3)
    
    echo "$(len my_list) items"
    echo "$(my_list --length) items"
    ```
    
- Dicts
    - dicts are just easier bash associative arrays
    
    ```bash
    my_dict<dict int str>=(a=1 b=2 c=3)
    ```
    
- Functions
    - inside of function may have optional curly bracket wrappers
    
    ```bash
    hello(
        # no type on arg means str; 
        msg... 
        # inside dots = repetable args to flag
        # outside dots = repeatable flag
        [-s --style <str>...]... 
        # no type on flag means boolean
        [-n]
        # grouped (same line) mean must be used together
        [--output<str> --path<str>] 
        # optional secondary flag
        [--input<str> [--path<str>]]
        # required flag
        --required
        # default
        --name<str="mike">
    ){
        echo "$msg with $style"
    }
    
    # no brackets required
    # semi colon if one liner with flags (delineates newline)
    world(name; -o)
        prt $name
    ```
    
- Commands
    
    ```bash
    # . in front means script command
    .cmd(input)
        # can be nested
        .subcmd(input)
    ```
    
- Builtins
    - built in compile/runtime type checking
    
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
    
- Conditional
    
    ```bash
    name=$1
    
    if $name = mike
        echo "hello"
    elif $name = mel
        echo "world"
    elif $name != fish
        echo "worm"
    else
        throw 1 "Bad Input"
    ```
    
- For / While Loops
    
    ```jsx
    for i in 1 2 3
    	echo $i
    	
    while true
    	echo "hello"
    ```
