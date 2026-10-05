# Command Grouping
* there are two types of command grouping:
  * subshell grouping `( ... )` 
  * command grouping `{ ... }`

## Subshell Grouping `( ... )`
* groups commands together and executes them in a subshell (child process)
* spaces are not required after `(` and before `)`

```bash
(echo "hello"; echo "world")
( echo "hello" )
```

### Use Cases
* safe execution of commands that modify the current shell environment
```bash
# Safe execution of commands that modify the current shell environment
(cd /tmp; echo "current directory: $(pwd)")
```

## Command Grouping `{ ... }`
* groups commands together and executes them in the current shell (same process)
* any changes made inside the group affect the current shell/script
* You must put a space after `{` and before `}`

```bash
# Correct syntax
{ echo "hello"; echo "world"; }

# Also correct (multi-line)
{
  echo "hello"
  echo "world"
}

# Incorrect: throws syntax errors
{echo "hello";}      # Missing space after {
{ echo "hello" }     # Missing semicolon before }
{ echo "hello" };    # Missing space before ;
```

### Use Cases
* directing multiple commands to a single pipe or file
```bash
# Efficient logging without spawning an extra process
{
  echo "=== Backup started at $(date) ==="
  tar -czf site_backup.tar.gz ./site
  echo "=== Backup completed ==="
} > backup.log 2>&1
```

