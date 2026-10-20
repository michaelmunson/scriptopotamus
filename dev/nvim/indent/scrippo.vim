if exists('b:did_indent')
  finish
endif
let b:did_indent = 1

setlocal indentexpr=GetScrippoIndent()
setlocal indentkeys=0{,0},0),0],!^F,o,O,0=else,0=elif
setlocal nosmartindent nocindent nolisp autoindent

let b:undo_indent = 'setlocal indentexpr< indentkeys< smartindent< cindent< lisp< autoindent<'

if exists('*GetScrippoIndent')
  finish
endif

let s:cpo_save = &cpo
set cpo&vim

" A line opens an indented block when it is a complete block header: a
" condition, `else`, a loop head, a macro/function/command header with its
" parameter list closed, a case arm without a body, or a lone `)`.
let s:opener = '\v^\s*%(%(if|elif|while|case)\s+\S.*|else|for\s+\w+\s+in\s+\S.*|\@\h\w*\(.*\)|%(\.|\.?\h[[:alnum:]_-]*)\(.*\)|[^[:space:]()#][^()]*\)|\))\s*$'
let s:open_params = '\v^\s*%(\@\h\w*|\.?\h[[:alnum:]_-]*|\.)\([^)]*$'

function! s:StripComment(line) abort
  return substitute(a:line, '\v%(^|\s)#.*$', '', '')
endfunction

function! GetScrippoIndent() abort
  let l:prev = prevnonblank(v:lnum - 1)
  if l:prev == 0
    return 0
  endif
  let l:prevline = s:StripComment(getline(l:prev))
  let l:ind = indent(l:prev)
  let l:current = getline(v:lnum)

  if l:current =~# '^\s*\%(else\|elif\)\>'
    if l:prevline =~# '^\s*\%(if\|elif\)\>'
      return l:ind
    endif
    let l:cur = l:ind
    let l:lnum = l:prev - 1
    while l:lnum > 0
      if getline(l:lnum) =~# '\S'
        let l:i = indent(l:lnum)
        if l:i < l:cur
          let l:cur = l:i
          if getline(l:lnum) =~# '^\s*\%(if\|elif\)\>'
            return l:i
          endif
        endif
      endif
      let l:lnum -= 1
    endwhile
    return max([0, l:ind - shiftwidth()])
  endif

  if l:prevline =~# s:opener || l:prevline =~# s:open_params
    let l:ind += shiftwidth()
  endif

  if l:current =~# '^\s*)'
    let l:ind = max([0, l:ind - shiftwidth()])
  endif
  return l:ind
endfunction

let &cpo = s:cpo_save
unlet s:cpo_save
