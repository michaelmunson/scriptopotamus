# Positional Parameters

```bash
# used for the example below
function get_num_parameters {
  echo $#;
}
```

### `$@`
* Expands to all positional parameters
* Each parameter is a separate argument
* Putting quotes around `$@` preserves spaces and special characters
```bash
set -- "a b" "c" "d"
get_num_parameters "$@"    # prints: 3
get_num_parameters $@      # prints: 4
```

### `$*`
* Expands to all positional parameters, but without the special treatment of spaces and special characters
```bash
set -- "a b" "c" "d"
get_num_parameters $*      # prints: 4
get_num_parameters "$*"    # prints: 1
```