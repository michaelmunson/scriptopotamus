# `set`
The `set` builtin is used to alter the shell's behavior, control execution options, and manipulate positional parameters.

## Common options
- `-e` : Exit immediately if a command exits with a non-zero status.
- `-u` : Treat unset variables as an error and exit immediately.
- `-o pipefail` : The return value of a pipeline is the status of the last command to exit with a non-zero status.
- `-x` : Print commands and their arguments as they are executed (useful for debugging).
- `-a` : Mark variables which are modified or created for export.
- `+<option>` : Turns the option OFF (i.e., `set +e` disables `-e`).

## Usage Examples
```bash
set -e            # Exit on error
set -u            # Treat unset variables as errors
set -a            # Automatically export all variables
set -x            # Print commands as they are executed (debug)
set -o pipefail   # Fail as soon as any command in a pipeline fails
set -euo pipefail # Common strict mode: error on first failure, unset vars, and pipeline errors

# To disable an option:
set +e            # Stop exiting on error
```

## Inspect current settings
```bash
set -o            # List current setting of all settable options
set -o posix      # Enable POSIX mode
```

## Manipulate positional parameters
```bash
set -- arg1 arg2  # Replace positional parameters with arg1 and arg2
echo "$1" "$2"
```
