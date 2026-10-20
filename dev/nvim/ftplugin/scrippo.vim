if exists('b:did_ftplugin')
  finish
endif
let b:did_ftplugin = 1

let s:cpo_save = &cpo
set cpo&vim

setlocal commentstring=#\ %s
setlocal comments=:#?,:#
setlocal formatoptions-=t
setlocal formatoptions+=croql
setlocal suffixesadd=.scrippo
setlocal omnifunc=v:lua.require'scrippo.complete'.omnifunc

let b:undo_ftplugin = 'setlocal commentstring< comments< formatoptions< suffixesadd< omnifunc<'

if has('nvim')
  lua require('scrippo').attach(vim.api.nvim_get_current_buf())
endif

let &cpo = s:cpo_save
unlet s:cpo_save
