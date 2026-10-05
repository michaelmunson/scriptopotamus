# Process Substitution
* allows you to pass the output of a command to another command as if it were a file
* there are two types of process substitution:
  * input process substitution `<(command)`
  * output process substitution `>(command)`

## Input Process Substitution `<(command)`
* allows you to pass the output of a command to another command as if it were a file
```bash
bat <(echo "hello") # bat expects a file
```
### Use Cases
* pass the output of a command to another command as if it were a file
```bash
# Compare files in a local directory vs a remote server
diff -u <(ls -1 /local/dir | sort) <(ssh server 'ls -1 /remote/dir' | sort)

# Compare active environment variables before and after sourcing a script
diff <(env | sort) <(bash -c 'source .env && env | sort')
```

## Output Process Substitution `>(command)`
* allows you to create a writable destination that feeds into a commands stdin
### Use Cases
```bash
# Route a server's log file output directly through grep or a formatter
my_server --log-file >(grep --line-buffered "ERROR" > error_only.log)
```

