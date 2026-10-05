# Redirection
* in unix, every running program opens with three default communication channels:
  * 0 stdin (standard input)
  * 1 stdout (standard output)
  * 2 stderr (standard error)
* redirection is the process of pointing these channels elsewhere

## Basic Redirection `>`, `>>`, `<`
* `>` (or `1>`) redirects stdout to a file
```bash
echo "hello" > file.txt # redirects stdout to file.txt
```
* `>>` (or `1>>`) redirects stdout to a file, appending to the end of the file
```bash
echo "hello" >> file.txt # redirects stdout to file.txt, appending to the end of the file
```
* `<` redirects stdin from a file
```bash
cat < file.txt # redirects stdin from file.txt
```

## Redirecting Errors
* errors bypass standard `>` output redirection
* to redirect errors, use `2>` or `2>>`
```bash
# Redirect only errors to a file (normal output still displays on screen)
find /etc -name "*.conf" 2> errors.log

# Append errors to an existing log
find /etc -name "*.conf" 2>> errors.log

# Silence errors entirely by sending them to the bitbucket
grep "pattern" /var/log/* 2> /dev/null
```
* both streams can be targeted in one command
```bash
my_script.sh > results.log 2> errors.log
```

## Merging Streams
* to merge stdout and stderr into a single stream, use `2>&1`
```bash
my_script.sh 2>&1 > output.log
```
* in modern bash, you can use `&>` as shorthand
```bash
my_script.sh &> output.log # equivalent to my_script.sh > output.log 2>&1
my_script.sh >& output.log # equivalent to my_script.sh > output.log 2>&1
```

## Advanced Redirection (Here Documents, Here Strings, and Closing Descriptors)
* `<< DELIMITER` is called a here document
  * it feeds a multiline block of text directly into a commands stdin
  ```bash
  cat << EOF
  snakgoat
  is a legend
  EOF
  ```
* `<<< STRING` is called a here string
  * it passes a string or expanded variable straight into a command's stdin without an extra echo or pipe
  ```bash
  cat <<< "snakgoat is a legend"
  ```
* `>&-` and `<&-` are called a closing descriptor
  * these completely shut down a communication stream
  ```bash
  exec 2>&- # close stderr
  exec 1>&- # close stdout
  ```