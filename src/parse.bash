#!/usr/bin/env bash

function parse {
	input=${1:?"parse: input required"}
	local exprs=("")
	local expr_idx=0
	for (( i=0; i<${#input}; i++ )); do
		char="${input:$i:1}"
		case "$char" in
    			[a-zA-Z0-9_]) 
				exprs[$expr_idx]="${exprs[expr_idx]}$char"
				;;
			\() 
				echo "(" 
				;;
			\)) 	
				echo ")" 
				;;
    			'<') 
				exprs[$expr_idx]="[var]::${exprs[expr_idx]}" 
				expr_idx=$((expr_idx+1))
				;;
    			'>') 
				exprs[$expr_idx]="[type]::${exprs[expr_idx]}"
				expr_idx=$((expr_idx+1))
				;;
    			' ') 
				echo "SPACE" 
				;;
  		esac
	done

	echo "${exprs[@]}"
}

parse "$1"
