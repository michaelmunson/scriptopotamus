# shellcheck disable=SC1091

SCRIPT_DIR=$(dirname "$0");
source "$SCRIPT_DIR/../../src/utils/utils.bash"
source "$SCRIPT_DIR/../test.bash"

function test_enforce_input {
  local inputs=("asd" "$UNDEFINED" "123");
  expect_equal $(enforce_input "${inputs[@]}") 0;
  
}
