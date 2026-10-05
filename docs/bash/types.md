# Bash Types (i.e `declare`)

* `-i`: integer:
  * Auto-evaluates arithmetic expressions on assignment
  ```bash
  declare -i x=1+2
  echo $x # 3
  ```
* `-a`: array:
  * Declares an indexed array variable
  ```bash
  declare -a x=(1 2 3)
  echo ${x[0]} # 1
  ```

* `-A`: associative array:
  * Declares an associative array variable (which is like a dictionary in other languages)
  ```bash
  declare -A x=([a]=1 ["b"]=2 [c]=3)
  echo ${x["a"]} # 1
  echo ${x[b]} # 2
  ```

* `-n`: nameref:
  * Creates a nameref variable that references another variable
  ```bash
  declare -n x=y
  x=1
  echo $y # 1
  ```

* `-r`: readonly:
  * Makes a variable read-only
  ```bash
  target="original value"
  declare -n ref=target

  echo "$ref"       # Output: original value

  ref="updated value"
  echo "$target"    # Output: updated value
  ```

* `-x`: export:
  * Exports a variable to the environment
  ```bash
  declare -x x=1
  echo $x # 1
  ```

* `-l`: lowercase:
  * Converts a variable to lowercase
  ```bash
  declare -l x=Hello
  echo $x # hello
  ```

* `-u`: uppercase:
  * Converts a variable to uppercase
  ```bash
  declare -u x=hello
  echo $x # HELLO
  ```

* `-p`: print:
  * Prints the value of a variable
  ```bash
  declare -p x
  echo $x # 1
  ```