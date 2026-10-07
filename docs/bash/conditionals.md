# Conditionals

### `if`
* 0 = true, 1 = false (weird i know)
* always use [[ ... ]] syntax
* use (( ... )) for arithmetic 
```bash
# grep exits 0 if a match is found, 1 if not
if grep -q "ERROR" app.log; then
  echo "Found an error!"
fi

name="Alice"

# String comparison and pattern match
if [[ $name == A* ]]; then
  echo "Name starts with A"
fi

# Regex matching (matches numbers)
version="v2.10.4"
if [[ $version =~ ^v[0-9]+\.[0-9]+ ]]; then
  echo "Valid version format"
fi

count=5

if (( count >= 5 && count < 10 )); then
  echo "Count is between 5 and 9"
fi
```

### `case`
* ;;⁠  = Standard terminator. Breaks out of the ⁠case⁠ statement once a match runs.
* ⁠;&⁠  = Fall-through. Continues execution into the next clause without testing its pattern.
* ⁠;;&⁠ = Resume testing. Continues testing subsequent patterns down the list.

```bash
file="archive.tar.gz"

case "$file" in
  *.tar.gz|*.tgz)
    tar -xzf "$file"
    ;;
  *.zip)
    unzip "$file"
    ;;
  *.txt)
    cat "$file"
    ;;
  *)
    echo "Unsupported format: $file"
    exit 1
    ;;
esac

```

```bash

```

```bash

```