function expect_equal {
  local expected="$1"
  local actual="$2"
  if [ "$expected" != "$actual" ]; then
    echo "Expected $expected but got $actual"
    return 1
  fi
  return 0
}