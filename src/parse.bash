FILE_PATH=$1

if [ -z "$FILE_PATH" ]; then
  echo "File path is required"
  exit 1
fi

function parse(){
  local text="$1"
  echo "Hello"
}

parse 