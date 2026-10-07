# Syntax & Expressions

### `bash`: no change needed
```bash
echo "asd" > file.txt
```

### `var`: declaration of a variable
```bash
# with type
my_num<int>
my_num<int>=
my_num<int> = 4

# no type
my_unknown=
```

### `math`: arithmetic expressions
```bash
my_float<float> = 3.14 + 8
```

### `cond`: conditionals
```bash
is_cool=true

if $is_cool
    echo coolio
else
    echo not coolio
```

### `for`: for loops
```bash
for i in $var
    echo $i
```

### `while`: while loops
```bash
while $var
    echo its true
```

### `func`: declaration of a function
```bash
# simple
my_func(name age<int>)
    echo "$name is $age years old"

# advanced
my_advanced_func(
    path
    -v<str>
    [
        --select
        --options<str>...
        [--multi]
    ] | [
        --confirm
    ]
)
    if $--select
        for i in $options
            echo $i
    else
        echo "nope"
```
