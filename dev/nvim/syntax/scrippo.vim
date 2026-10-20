if exists('b:current_syntax')
  finish
endif

let s:cpo_save = &cpo
set cpo&vim

syn case match
syn sync fromstart

syn keyword scrippoTodo contained TODO FIXME XXX NOTE
syn match scrippoComment /\%(^\|\s\)\zs#.*$/ contains=scrippoTodo,@Spell
syn match scrippoDocComment /\%(^\|[[:space:];]\)\zs#?.*$/ contains=scrippoTodo,@Spell
syn match scrippoShebang /\%1l^#!.*$/

" ---------------------------------------------------------------- variables
syn match scrippoSpecialVar /\$[0-9@#?*!$-]/
syn match scrippoVariable /\$\h\w*/
syn match scrippoOptionRef /\$--\=\h[[:alnum:]_-]*/
syn region scrippoIndexedVar matchgroup=scrippoVariable start=/\$\h\w*\[/ end=/\]/ oneline contains=scrippoVariable,scrippoNumber,scrippoIndexKey
syn match scrippoIndexKey /[^[:space:]$\]]\+/ contained
syn region scrippoBracedVar matchgroup=scrippoVariable start=/\${/ end=/}/ contains=scrippoDoubleString,scrippoSingleString,scrippoSubstitution,scrippoVariable,scrippoSpecialVar,scrippoExpansionOp
syn match scrippoExpansionOp /[#%/^,:@!*?+=-]\+/ contained

" ---------------------------------------------------------------- strings
syn match scrippoEscape /\\[\\$`"\n]/ contained
syn match scrippoAnsiEscape /\\\%([abeEfnrtv\\'"?]\|[0-7]\{1,3}\|x\x\{1,2}\|u\x\{1,4}\|U\x\{1,8}\|c.\)/ contained
syn region scrippoDoubleString start=/"/ skip=/\\\\\|\\"/ end=/"/ contains=scrippoEscape,scrippoArithmetic,scrippoSubstitution,scrippoBacktick,scrippoBracedVar,scrippoIndexedVar,scrippoOptionRef,scrippoVariable,scrippoSpecialVar,@Spell
syn region scrippoSingleString start=/'/ end=/'/ contains=@Spell
syn region scrippoAnsiString start=/\$'/ skip=/\\\\\|\\'/ end=/'/ contains=scrippoAnsiEscape

" ---------------------------------------------------------------- expressions
syn match scrippoEchoArrow /\%(\w\|-\)\@<!->/
syn match scrippoMacro /\%(\w\|\$\)\@<!@\h\w*/
syn match scrippoFlow /\%(\w\|[$.-]\)\@<!\%(return\|exit\|break\|continue\)\%(\w\|-\)\@!/
syn match scrippoBuiltin /\%(\w\|[$./-]\)\@<!\%(echo\|printf\|read\|cd\|source\|shift\|local\|declare\|export\|readonly\|unset\|set\|eval\|exec\|test\|trap\|wait\|mapfile\|let\)\%(\w\|-\)\@!/
syn match scrippoBoolean /\%(\w\|[$.-]\)\@<!\%(true\|false\)\%(\w\|-\)\@!/
syn match scrippoNumber /\%(\w\|[$.-]\)\@<!-\=\%(\d\+\.\=\d*\|\.\d\+\)\%(\w\|[.-]\)\@!/
syn match scrippoOption /\%(^\|[[:space:](]\)\@<=--\=\a[[:alnum:]_-]*/
syn match scrippoRedirect /\d*\%(&>>\=\|>>\|>&\d*-\=\|<<<\|<<-\=\|<&\|>\|<\)/
syn match scrippoLogical /&&\|||\|[|;&!]/
syn match scrippoComparison /==\|!=\|=\~\|=/
syn match scrippoArithOp /\s\zs[-+*/%^]\ze\s/
syn match scrippoImportAs /\%(^\|\s\)\zs--as\>/ nextgroup=scrippoImportScope skipwhite
syn match scrippoImportScope /\.\=\h[[:alnum:]_-]*/ contained

syn region scrippoSubstitution matchgroup=scrippoSubstDelim start=/\$(/ end=/)/ contains=@scrippoExpr,scrippoSubstCall
syn match scrippoSubstCall /\%(\$(\s*\)\@<=\h[[:alnum:]_-]*/ contained
syn region scrippoArithmetic matchgroup=scrippoSubstDelim start=/\$((/ end=/))/ contains=@scrippoExpr
syn region scrippoBacktick matchgroup=scrippoSubstDelim start=/`/ end=/`/ contains=@scrippoExpr
syn region scrippoParens matchgroup=scrippoDelimiter start=/(/ end=/)/ contains=@scrippoExpr,scrippoKeyValue
syn match scrippoKeyValue /\%(\s\|(\)\@<=[^[:space:]=()"'$]\+\ze=/ contained

syn cluster scrippoExpr contains=scrippoComment,scrippoDocComment,scrippoEchoArrow,scrippoAnsiString,scrippoDoubleString,scrippoSingleString,scrippoArithmetic,scrippoSubstitution,scrippoBacktick,scrippoBracedVar,scrippoIndexedVar,scrippoOptionRef,scrippoVariable,scrippoSpecialVar,scrippoFlow,scrippoMacro,scrippoBuiltin,scrippoBoolean,scrippoNumber,scrippoOption,scrippoRedirect,scrippoLogical,scrippoComparison,scrippoArithOp,scrippoParens,scrippoImportAs

" ---------------------------------------------------------------- types
syn keyword scrippoTypeConst contained const
syn keyword scrippoTypePrimitive contained int integer float str string bool boolean any
syn keyword scrippoTypeCollection contained list dict array
syn match scrippoTypeName /\h\w*/ contained
syn match scrippoTypeDefaultOp /=/ contained
syn match scrippoTypeLiteral /[^[:space:])}"'>]\+/ contained
syn region scrippoTypeLiterals matchgroup=scrippoDelimiter start=/[({]/ end=/[)}]/ contained contains=scrippoDoubleString,scrippoSingleString,scrippoNumber,scrippoBoolean,scrippoTypeLiteral
syn region scrippoTypeAnn matchgroup=scrippoDelimiter start=/</ end=/>/ contained contains=scrippoTypeConst,scrippoTypePrimitive,scrippoTypeCollection,scrippoTypeName,scrippoTypeLiterals,scrippoTypeDefaultOp,scrippoDoubleString,scrippoSingleString,scrippoNumber,scrippoBoolean,scrippoVariable,scrippoBracedVar nextgroup=scrippoAssignOp

" ---------------------------------------------------------------- parameters
syn match scrippoParamName /\%(\w\|-\)\@<!\h[[:alnum:]_-]*/ contained
syn match scrippoParamRest /\%(\w\|\*\|-\)\@<!\*\h[[:alnum:]_-]*/ contained
syn match scrippoParamOption /\%(\w\|-\)\@<!--\=\h[[:alnum:]_-]*/ contained
syn match scrippoParamRepeat /\.\.\./ contained
syn match scrippoParamExclusive /|/ contained
syn match scrippoParamAlias /\// contained
syn match scrippoParamOptional /[][]/ contained
syn match scrippoParamMacro /@\h\w*/ contained
syn region scrippoParams matchgroup=scrippoDelimiter start=/(/ end=/)/ contained contains=scrippoComment,scrippoDocComment,scrippoTypeAnn,scrippoParamMacro,scrippoParamRepeat,scrippoParamExclusive,scrippoParamAlias,scrippoParamOptional,scrippoParamRest,scrippoParamOption,scrippoParamName

syn match scrippoCommandDef /^\s*\zs\.\%(\h[[:alnum:]_-]*\)\=\ze(/ nextgroup=scrippoParams
syn match scrippoMacroDef /^\s*\zs@\h\w*\ze(/ nextgroup=scrippoParams
syn match scrippoFunctionDef /^\s*\zs\%(\%(if\|elif\|else\|for\|while\|case\)\%(\w\|-\)\@!\)\@!\h[[:alnum:]_-]*\ze(/ nextgroup=scrippoParams

" ---------------------------------------------------------------- declarations
syn match scrippoTypedVar /^\s*\zs\h\w*\ze<[^>]*>\s*\%(=\|$\|#\)/ nextgroup=scrippoTypeAnn
syn match scrippoAssignment /^\s*\zs\h\w*\ze=/ nextgroup=scrippoAssignOp
syn match scrippoAssignOp /=/ contained

" ---------------------------------------------------------------- control flow
syn match scrippoTestOp /\%(^\|\s\)\@<=!\=-\%([a-zA-Z]\|eq\|ne\|lt\|le\|gt\|ge\|nt\|ot\|ef\)\ze\%(\s\|$\)/ contained
syn match scrippoElse /;\s*\zselse\%(\w\|-\)\@!/ contained
syn region scrippoCondition matchgroup=scrippoConditional start=/^\s*\zs\%(if\|elif\)\ze\%(\s\|$\)/ end=/$/ oneline keepend contains=@scrippoExpr,scrippoTestOp,scrippoElse
syn region scrippoLoopCondition matchgroup=scrippoRepeat start=/^\s*\zswhile\ze\%(\s\|$\)/ end=/$/ oneline keepend contains=@scrippoExpr,scrippoTestOp,scrippoElse
syn match scrippoElseLine /^\s*\zselse\%(\w\|-\)\@!/
syn match scrippoFor /^\s*\zsfor\ze\s/ nextgroup=scrippoForVar skipwhite
syn match scrippoForVar /\h\w*/ contained nextgroup=scrippoForIn skipwhite
syn match scrippoForIn /in\%(\w\|-\)\@!/ contained
syn match scrippoCase /^\s*\zscase\ze\%(\s\|$\)/ nextgroup=scrippoCaseIn
syn match scrippoCaseIn /\s\zsin\s*$/ contained
syn match scrippoCasePipe /|/ contained
syn match scrippoCaseParen /)/ contained
syn match scrippoCaseTerminator /;;&\|;;\|;&/
syn match scrippoBlockCloser /^\s*\zs\%(fi\|done\|then\|do\|esac\)\ze\%(\s\|;\|$\)/

" Command calls are matched last so that keywords and builtins above keep priority.
syn match scrippoCommandCall /^\s*\zs\%(\%(if\|elif\|else\|for\|while\|case\|fi\|done\|then\|do\|esac\|true\|false\|echo\|printf\|read\|cd\|source\|shift\|local\|declare\|export\|readonly\|unset\|set\|eval\|exec\|test\|trap\|wait\|mapfile\|let\|return\|exit\|break\|continue\)\%(\w\|-\)\@!\)\@!\h[[:alnum:]_-]*\ze\%(\s\|$\|;\)/

syn match scrippoCaseArm /^\s*\zs\%("[^"]*"\|'[^']*'\|[^()#"' \t]\)\%("[^"]*"\|'[^']*'\|[^()#"']\)*\ze)/ contains=scrippoCasePipe,scrippoDoubleString,scrippoSingleString,scrippoVariable nextgroup=scrippoCaseParen

" ---------------------------------------------------------------- highlighting
hi def link scrippoShebang PreProc
hi def link scrippoTodo Todo
hi def link scrippoComment Comment
hi def link scrippoDocComment SpecialComment

hi def link scrippoVariable Identifier
hi def link scrippoBracedVar Identifier
hi def link scrippoSpecialVar Special
hi def link scrippoOptionRef Special
hi def link scrippoIndexKey String
hi def link scrippoExpansionOp Operator

hi def link scrippoEscape SpecialChar
hi def link scrippoAnsiEscape SpecialChar
hi def link scrippoDoubleString String
hi def link scrippoSingleString String
hi def link scrippoAnsiString String

hi def link scrippoEchoArrow Statement
hi def link scrippoMacro Macro
hi def link scrippoFlow Statement
hi def link scrippoBuiltin Function
hi def link scrippoBoolean Boolean
hi def link scrippoNumber Number
hi def link scrippoOption Special
hi def link scrippoRedirect Operator
hi def link scrippoLogical Operator
hi def link scrippoComparison Operator
hi def link scrippoArithOp Operator
hi def link scrippoImportAs Include
hi def link scrippoImportScope Function
hi def link scrippoSubstDelim Delimiter
hi def link scrippoSubstCall Function
hi def link scrippoDelimiter Delimiter
hi def link scrippoKeyValue Label

hi def link scrippoTypeConst StorageClass
hi def link scrippoTypePrimitive Type
hi def link scrippoTypeCollection Type
hi def link scrippoTypeName Type
hi def link scrippoTypeDefaultOp Operator
hi def link scrippoTypeLiteral Constant

hi def link scrippoParamName Identifier
hi def link scrippoParamRest Identifier
hi def link scrippoParamOption Special
hi def link scrippoParamRepeat Operator
hi def link scrippoParamExclusive Operator
hi def link scrippoParamAlias Operator
hi def link scrippoParamOptional Delimiter
hi def link scrippoParamMacro Macro

hi def link scrippoCommandDef Function
hi def link scrippoMacroDef Macro
hi def link scrippoFunctionDef Function
hi def link scrippoTypedVar Identifier
hi def link scrippoAssignment Identifier
hi def link scrippoAssignOp Operator

hi def link scrippoTestOp Operator
hi def link scrippoElse Conditional
hi def link scrippoElseLine Conditional
hi def link scrippoConditional Conditional
hi def link scrippoRepeat Repeat
hi def link scrippoFor Repeat
hi def link scrippoForVar Identifier
hi def link scrippoForIn Repeat
hi def link scrippoCase Conditional
hi def link scrippoCaseIn Conditional
hi def link scrippoCaseArm String
hi def link scrippoCasePipe Operator
hi def link scrippoCaseParen Operator
hi def link scrippoCaseTerminator Delimiter
hi def link scrippoBlockCloser Error
hi def link scrippoCommandCall Function

let b:current_syntax = 'scrippo'

let &cpo = s:cpo_save
unlet s:cpo_save
