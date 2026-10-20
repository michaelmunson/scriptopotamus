augroup scrippo_ftdetect
  autocmd!
  autocmd BufRead,BufNewFile *.scrippo,*.scriptopotamus setfiletype scrippo
  autocmd BufRead,BufNewFile * if getline(1) =~# '^#!.*\<\%(scrippo\|scriptopotamus\)\>' | setfiletype scrippo | endif
augroup END
