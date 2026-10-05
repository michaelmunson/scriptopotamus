# Parameter Expansion
- Used to inspect, transform, and evaluate the content of a variable before a command runs

```bash
# used in examples below
get_num_params(){ echo "$#"; }
```

### Handling Empty/Unset Variables

- `${NAME:-snakgoat}`: If `NAME` is unset or empty, use "snakgoat"
- `${NAME:+snakgoat}`: If `NAME` is set, use "snakgoat"
- `${NAME:=snakgoat}`: If `NAME` is unset or empty, set it to "snakgoat"
- `${NAME:?snakgoat}`: If `NAME` is unset or empty, print "snakgoat" and exit

### String Trimming: Pattern Removal

- `#` strips the shortest matching pattern from the beginning of the string
- `##` strips the longest matching pattern from the beginning of the string
- `%` strips the shortest matching pattern from the end of the string
- `%%` strips the longest matching pattern from the end of the string
  > *`#` is on the left side of the keyboard (therefore strip left side of the string)*
  > *`%` is on the right side of the keyboard (therefore strip right side of the string)*

```bash
FILE_PATH="/path/to/archive.tar.gz"
echo "${FILE_PATH#*/}" # prints: path/to/archive.tar.gz
echo "${FILE_PATH##*/}" # prints: archive.tar.gz
echo "${FILE_PATH%.*}" # prints: /path/to/archive.tar
echo "${FILE_PATH%%.*}" # prints: /path/to/archive
```

### Search and Replace

- `var/pattern/replacement` replace first match
- `var//pattern/replacement` replace all matches
- `var/#pattern/replacement` replace first match only if it is at the beginning of the string
- `var/%pattern/replacement` replace first match only if it is at the end of the string
- `var/pattern` remove first match
- `var//pattern` remove all matches
- `var/#pattern` remove first match only if it is at the beginning of the string
- `var/%pattern` remove first match only if it is at the end of the string

```bash
TEXT="goat fish horse goat"
echo "${TEXT/goat/snakgoat}" # prints: snakgoat fish horse goat
echo "${TEXT//goat/snakgoat}" # prints: snakgoat fish horse snakgoat
echo "${TEXT/#goat/snakgoat}" # prints: snakgoat fish horse goat
echo "${TEXT/%goat/snakgoat}" # prints: goat fish horse snakgoat
echo "${TEXT/goat}" # prints: fish horse goat
echo "${TEXT//goat}" # prints: fish horse
echo "${TEXT/#goat}" # prints: goat fish horse goat
echo "${TEXT/%goat}" # prints: goat fish horse goat
```

### Substring Slicing & Length

- `${var:offset:length}` slice the string from `offset` to `length` (end index = `offset` + `length`)
- `${var:start}` slice the string from `start` index to the end of the string
- `${var: -length}` slice the string from the end of the string to `length` *! don't forget the space before the length*
- `${#var}` length of the string

```bash
NAME="goat fish horse goat"
echo "${NAME:0:3}" # prints: goa
echo "${NAME:5}" # prints: fish horse goat
echo "${NAME: -9}" # prints: horse
echo "${#NAME}" # prints: 20
```

### Case Modification
* `${var^^}` convert to uppercase
* `${var,,}` convert to lowercase
* `${var^}` convert the first character to uppercase
* `${var,}` convert the first character to lowercase

```bash
NAME="snakgoat"
echo "${NAME^^}" # prints: SNAKGOAT
echo "${NAME,,}" # prints: snakgoat
echo "${NAME^}" # prints: Snakgoat
echo "${NAME,}" # prints: snakgoat
```

### Arrays
* `${ARRAY[index]}` get the element at the `index`
* `${ARRAY[@]}` expand all elements as separate distinct strings
* `${ARRAY[*]}` expand all elements in a single string (combined into a single string with IFS)
* `${#ARRAY[@]}` length of the array
* `${!ARRAY[@]}` expand all indices of the array

```bash
ARRAY=(goat fish horse goat)
echo "${ARRAY[0]}" # prints: goat
echo "${ARRAY[@]}" # prints: goat fish horse goat
echo "${ARRAY[*]}" # prints: goat fish horse goat
echo "${!ARRAY[@]}" # prints: 0 1 2 3
echo "${ARRAY[@]:1:2}" # prints: fish horse

get_num_params "${ARRAY[@]}" # prints: 3
get_num_params "${ARRAY[*]}" # prints: 1
```