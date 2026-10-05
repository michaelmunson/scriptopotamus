#!/usr/bin/env bash
USAGE="
Usage: parse.bash <file_path>
"

FILE_PATH=${1:?$USAGE}

function parse(){
  local file_path="$1"
  local text=$(cat "$file_path")
  echo "Hello"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  parse "$(cat "$FILE_PATH")"
fi

