#                     Bash Dictionary Utility                         #
#######################################################################
#
# This function creates a dictionary variable in the current scope.
#
# Usage: dict <key> <value>
# Example: dict "name" "John"
# Output: {"name": "John"}
#
# ---------------------------------------------------------------------

declare -A MASTER_DICT=()

function dict {
  declare -A 
}