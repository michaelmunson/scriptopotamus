
# Expr Classification

function is_var {
	local input=${1:?"is_var: Arg Required"}
	if [[ $input =~ ^[a-zA-Z_][a-zA-Z0-9_]*< ]]; then
		return 0
	fi
	return 1
}


