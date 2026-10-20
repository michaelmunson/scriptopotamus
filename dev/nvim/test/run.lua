-- Run with: nvim --headless -u NONE -l dev/nvim/test/run.lua
local here = debug.getinfo(1, 'S').source:sub(2)
local root = vim.fn.fnamemodify(here, ':p:h:h')
vim.opt.runtimepath:prepend(root)
vim.cmd('filetype plugin indent on')
vim.cmd('syntax on')

local failures = 0
local total = 0

local function check(name, ok, detail)
  total = total + 1
  if not ok then
    failures = failures + 1
    io.stderr:write('FAIL ' .. name .. (detail and (': ' .. detail) or '') .. '\n')
  end
end

local function eq(name, actual, expected)
  check(name, vim.deep_equal(actual, expected), vim.inspect(actual) .. ' ~= ' .. vim.inspect(expected))
end

local function open(lines, name)
  vim.cmd('enew!')
  local buf = vim.api.nvim_get_current_buf()
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  if name then
    vim.api.nvim_buf_set_name(buf, name)
  end
  vim.bo[buf].filetype = 'scrippo'
  vim.cmd('syntax sync fromstart')
  return buf
end

local function group_at(lnum, text, occurrence)
  local line = vim.fn.getline(lnum)
  local col = 0
  for _ = 1, occurrence or 1 do
    col = line:find(text, col + 1, true)
    if not col then
      return nil
    end
  end
  return vim.fn.synIDattr(vim.fn.synID(lnum, col, 1), 'name')
end

-- ---------------------------------------------------------------- filetype
do
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, 'p')
  vim.cmd('edit ' .. dir .. '/demo.scrippo')
  eq('ftdetect extension', vim.bo.filetype, 'scrippo')
  vim.cmd('enew!')
  vim.fn.writefile({ '#!/usr/bin/env scrippo', 'echo hi' }, dir .. '/tool')
  vim.cmd('edit ' .. dir .. '/tool')
  eq('ftdetect shebang', vim.bo.filetype, 'scrippo')
  eq('commentstring', vim.bo.commentstring, '# %s')
  check('omnifunc set', vim.bo.omnifunc:find('scrippo.complete', 1, true) ~= nil, vim.bo.omnifunc)
end

-- ---------------------------------------------------------------- syntax
do
  open({
    '#!/usr/bin/env scrippo',
    '@import scrippo/std',
    '@import "./x.scrippo" --as .demo',
    '#? documented',
    '# plain comment',
    'PI<const float>=3.14',
    'name="hi $USER ${HOME} $(whoami) $--loud"',
    '.greet(name [-s / --style<str="x">] files...) #? hi',
    '  if $loud && -f $name',
    '    -> "HELLO $name"',
    '  else',
    '    -> \'x\'',
    '  for i in $list',
    '    echo $i',
    '  case $str',
    '    hello | hi) -> "Hi"',
    '    *) -> "?" ;;',
    '@say_hi()',
    '  -> "hi"',
    '@say_hi',
    'build_it(a<int>)',
    '  prt -s red "x"',
  })
  eq('shebang', group_at(1, '#!'), 'scrippoShebang')
  eq('macro call in import', group_at(2, '@import'), 'scrippoMacro')
  eq('import --as', group_at(3, '--as'), 'scrippoImportAs')
  eq('import scope', group_at(3, '.demo'), 'scrippoImportScope')
  eq('doc comment', group_at(4, '#?'), 'scrippoDocComment')
  eq('comment', group_at(5, '# plain'), 'scrippoComment')
  eq('typed var', group_at(6, 'PI'), 'scrippoTypedVar')
  eq('const', group_at(6, 'const'), 'scrippoTypeConst')
  eq('primitive type', group_at(6, 'float'), 'scrippoTypePrimitive')
  eq('number', group_at(6, '3.14'), 'scrippoNumber')
  eq('assignment', group_at(7, 'name'), 'scrippoAssignment')
  eq('string', group_at(7, '"hi'), 'scrippoDoubleString')
  eq('variable in string', group_at(7, '$USER'), 'scrippoVariable')
  eq('braced var', group_at(7, 'HOME'), 'scrippoBracedVar')
  eq('subst call', group_at(7, 'whoami'), 'scrippoSubstCall')
  eq('option ref', group_at(7, '$--loud'), 'scrippoOptionRef')
  eq('command def', group_at(8, '.greet'), 'scrippoCommandDef')
  eq('param name', group_at(8, 'name'), 'scrippoParamName')
  eq('param option', group_at(8, '-s'), 'scrippoParamOption')
  eq('param alias', group_at(8, '/'), 'scrippoParamAlias')
  eq('param type', group_at(8, 'str'), 'scrippoTypePrimitive')
  eq('param repeat', group_at(8, '...'), 'scrippoParamRepeat')
  eq('doc comment after params', group_at(8, '#? hi'), 'scrippoDocComment')
  eq('if', group_at(9, 'if'), 'scrippoConditional')
  eq('test operator', group_at(9, '-f'), 'scrippoTestOp')
  eq('echo arrow', group_at(10, '->'), 'scrippoEchoArrow')
  eq('else', group_at(11, 'else'), 'scrippoElseLine')
  eq('single string', group_at(12, "'x'"), 'scrippoSingleString')
  eq('for', group_at(13, 'for'), 'scrippoFor')
  eq('for var', group_at(13, 'i'), 'scrippoForVar')
  eq('for in', group_at(13, 'in'), 'scrippoForIn')
  eq('builtin', group_at(14, 'echo'), 'scrippoBuiltin')
  eq('case', group_at(15, 'case'), 'scrippoCase')
  eq('case arm', group_at(16, 'hello'), 'scrippoCaseArm')
  eq('wildcard case arm', group_at(17, '*'), 'scrippoCaseArm')
  eq('case terminator', group_at(17, ';;'), 'scrippoCaseTerminator')
  eq('macro def', group_at(18, '@say_hi'), 'scrippoMacroDef')
  eq('macro use', group_at(20, '@say_hi'), 'scrippoMacro')
  eq('function def', group_at(21, 'build_it'), 'scrippoFunctionDef')
  eq('command call', group_at(22, 'prt'), 'scrippoCommandCall')
end

-- ---------------------------------------------------------------- analyzer
local analyzer = require('scrippo.analyzer')
do
  local leaves = analyzer.parse_params('name [-s / --style<str="x">] [--loud] files... *rest -v<str>')
  eq('param count', #leaves, 6)
  eq('arg name', leaves[1].name, 'name')
  eq('alias flags', leaves[2].flags, { '-s', '--style' })
  eq('alias var', leaves[2].var, 'style')
  eq('rest', leaves[5].rest, true)
  eq('short flag var', leaves[6].var, 'v')
end

-- ---------------------------------------------------------------- completion
local complete = require('scrippo.complete')

local function labels(result)
  local out = {}
  for _, item in ipairs(result and result.items or {}) do
    out[#out + 1] = item.label
  end
  return out
end

local function has(list, value)
  return vim.tbl_contains(list, value)
end

local SCRIPT = {
  '@import scrippo/std/io',
  'LIMIT<const int>=10',
  'greeting="hi"',
  '@say_hi()',
  '  -> "hi"',
  '.greet(name [-l / --loud] [--times<int=1>]) #? hi',
  '  -> $name',
  '  .shout(text)',
  '    -> $text',
  'helper(a b [--force])',
  '  -> $a',
  '',
}

local function at(lines, row, col, opts)
  return complete.complete(lines, row, col, opts)
end

local function with_line(text)
  local lines = vim.deepcopy(SCRIPT)
  lines[#lines] = text
  return lines, #lines, #text
end

do
  local r = at(with_line('@'))
  check('macro items', r and has(labels(r), 'import') and has(labels(r), 'usage') and has(labels(r), 'say_hi'), vim.inspect(labels(r)))
  eq('macro start', r.start, 1)

  r = at(with_line('  prt "x" && @sa'))
  eq('macro prefix start', r.start, #'  prt "x" && @')

  r = at(with_line('x=$'))
  check('top-level variables', r and has(labels(r), 'LIMIT') and has(labels(r), 'greeting'), vim.inspect(labels(r)))
  check('no params outside scope', r and not has(labels(r), 'name'), vim.inspect(labels(r)))

  local lines = vim.deepcopy(SCRIPT)
  lines[7] = '  -> $na'
  r = at(lines, 7, #lines[7])
  check('param in scope', r and has(labels(r), 'name') and has(labels(r), 'loud') and has(labels(r), 'times'), vim.inspect(labels(r)))

  lines[7] = '  -> $--'
  r = at(lines, 7, #lines[7])
  check('option refs', r and has(labels(r), '--loud') and has(labels(r), '-l') and has(labels(r), '--times'), vim.inspect(labels(r)))
  eq('option ref start', r.start, #'  -> $')

  lines[9] = '    -> ${te'
  r = at(lines, 9, #lines[9])
  check('nested scope params', r and has(labels(r), 'text') and has(labels(r), 'name'), vim.inspect(labels(r)))

  r = at(with_line('count<in'))
  check('declaration types', r and has(labels(r), 'int') and has(labels(r), 'const'), vim.inspect(labels(r)))
  eq('type start', r.start, #'count<')

  r = at(with_line('count<const '))
  check('type after const', r and has(labels(r), 'float') and not has(labels(r), 'const'), vim.inspect(labels(r)))

  r = at(with_line('count<int '))
  eq('no type after type', #labels(r), 0)

  r = at(with_line('@import scrippo/'))
  check('import packages', r and has(labels(r), 'scrippo/std') and has(labels(r), 'scrippo/std/io'), vim.inspect(labels(r)))
  eq('import start', r.start, #'@import ')

  r = at(with_line('@import "scrippo/std/io" --'))
  eq('import --as', labels(r), { '--as' })

  r = at(with_line('@import scrippo/std --as '))
  check('import scope', r and has(labels(r), '.command'), vim.inspect(labels(r)))

  r = at(with_line(''))
  check('command position', r and has(labels(r), 'greet') and has(labels(r), 'helper') and has(labels(r), 'prt') and has(labels(r), '@import') and has(labels(r), 'if') and has(labels(r), 'for'), vim.inspect(labels(r)))
  check('std not imported', r and not has(labels(r), 'list') and not has(labels(r), 'here'), vim.inspect(labels(r)))
  check('nested command hidden at top', r and not has(labels(r), 'shout'), vim.inspect(labels(r)))

  r = at(with_line('if gr'))
  check('after if', r and has(labels(r), 'greet') and not has(labels(r), 'if'), vim.inspect(labels(r)))

  r = at(with_line('x=$(hel'))
  check('substitution', r and has(labels(r), 'helper') and not has(labels(r), 'for'), vim.inspect(labels(r)))

  r = at(with_line('greet '))
  eq('sub commands', labels(r), { 'shout' })

  r = at(with_line('greet --'))
  check('command flags', r and has(labels(r), '--loud') and has(labels(r), '--times'), vim.inspect(labels(r)))

  r = at(with_line('greet --loud --'))
  check('used flag removed', r and not has(labels(r), '--loud') and not has(labels(r), '-l') and has(labels(r), '--times'), vim.inspect(labels(r)))

  r = at(with_line('helper x y --'))
  eq('function flags', labels(r), { '--force' })

  r = at(with_line('prt "x" -'))
  check('std flags', r and has(labels(r), '--style') and has(labels(r), '-j'), vim.inspect(labels(r)))

  eq('no completion in comment', at(with_line('# @im')), nil)
  eq('no completion in single quote', at(with_line("echo 'foo $")), nil)
  eq('no completion for plain args', at(with_line('echo hello wor')), nil)

  lines = vim.deepcopy(SCRIPT)
  lines[#lines] = 'wide(a<in'
  r = at(lines, #lines, #lines[#lines])
  check('param header types', r and has(labels(r), 'int'), vim.inspect(labels(r)))
  lines[#lines] = 'wide(a b'
  r = at(lines, #lines, #lines[#lines])
  eq('no functions inside header', r, nil)
end

-- ---------------------------------------------------------------- local imports
do
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir .. '/lib/deep', 'p')
  vim.fn.mkdir(dir .. '/.scrippo/lib', 'p')
  vim.fn.writefile({ '' }, dir .. '/lib/utils.scrippo')
  vim.fn.writefile({ '' }, dir .. '/lib/deep/more.scrippo')
  vim.fn.writefile({ '' }, dir .. '/.scrippo/lib/mylib.scrippo')
  local text = '@import ./li'
  local r = at({ text }, 1, #text, { import_dir = dir })
  check('local files', r and has(labels(r), './lib/utils.scrippo') and has(labels(r), './lib/deep/more.scrippo'), vim.inspect(labels(r)))

  text = '@import my'
  r = at({ text }, 1, #text, { import_dir = dir })
  check('project packages', r and has(labels(r), 'mylib'), vim.inspect(labels(r)))

  text = '@import "./lib/ut'
  r = at({ text }, 1, #text, { import_dir = dir })
  eq('quoted local import', r and r.items[1].insert, './lib/utils.scrippo"')
end

-- ---------------------------------------------------------------- omnifunc
do
  local buf = open(vim.list_extend(vim.deepcopy(SCRIPT), {}))
  vim.api.nvim_buf_set_lines(buf, #SCRIPT - 1, -1, false, { '@im' })
  vim.api.nvim_win_set_cursor(0, { #SCRIPT, 3 })
  eq('omnifunc findstart', complete.omnifunc(1, ''), 1)
  local items = complete.omnifunc(0, 'im')
  eq('omnifunc filter', #items, 1)
  eq('omnifunc word', items[1].word, 'import')

  vim.api.nvim_buf_set_lines(buf, #SCRIPT - 1, -1, false, { 'echo hello' })
  vim.api.nvim_win_set_cursor(0, { #SCRIPT, 10 })
  eq('omnifunc cancel', complete.omnifunc(1, ''), -3)

  vim.api.nvim_buf_set_lines(buf, #SCRIPT - 1, -1, false, { 'gr' })
  vim.api.nvim_win_set_cursor(0, { #SCRIPT, 2 })
  eq('omnifunc command start', complete.omnifunc(1, ''), 0)
  items = complete.omnifunc(0, 'gr')
  check('omnifunc command', #items == 1 and items[1].word == 'greet' and items[1].kind == 'command', vim.inspect(items))
end

-- ---------------------------------------------------------------- cmp source
do
  local source = require('cmp_scrippo').new()
  local buf = open(vim.deepcopy(SCRIPT))
  vim.api.nvim_buf_set_lines(buf, #SCRIPT - 1, -1, false, { '  @us' })
  local result
  source:complete({
    context = { cursor = { row = #SCRIPT, col = 6 }, cursor_before_line = '  @us', bufnr = buf },
  }, function(res)
    result = res
  end)
  local import
  for _, item in ipairs(result.items) do
    if item.label == 'usage' then
      import = item
    end
  end
  check('cmp item', import ~= nil and import.insertTextFormat == 2, vim.inspect(result))
  eq('cmp range start', import.textEdit.range.start.character, 3)
end

-- ---------------------------------------------------------------- indent
do
  local function indent_for(lines, typed)
    local buf = open(lines)
    vim.bo[buf].shiftwidth = 2
    vim.bo[buf].expandtab = true
    vim.api.nvim_buf_set_lines(buf, #lines, #lines, false, { typed })
    vim.api.nvim_win_set_cursor(0, { #lines + 1, 0 })
    vim.cmd(string.format('let v:lnum = %d', #lines + 1))
    return vim.fn.eval('GetScrippoIndent()')
  end
  eq('indent after if', indent_for({ 'if $x' }, ''), 2)
  eq('indent after command header', indent_for({ '.greet(name)' }, ''), 2)
  eq('indent after else', indent_for({ 'if $x', '  a', 'else' }, ''), 2)
  eq('keep indent', indent_for({ 'if $x', '  a' }, ''), 2)
  eq('else aligns with if', indent_for({ 'if $x', '  a' }, 'else'), 0)
  eq('nested else', indent_for({ 'if $x', '  if $y', '    a' }, 'else'), 2)
  eq('open params', indent_for({ '.greet(' }, ''), 2)
  eq('closing paren', indent_for({ '.greet(', '  name' }, ')'), 0)
  eq('body after params', indent_for({ '.greet(', '  name', ')' }, ''), 2)
  eq('inline body', indent_for({ 'f(a) -> echo $a' }, ''), 0)
end

io.stderr:write(string.format('%d checks, %d failed\n', total, failures))
vim.cmd(failures == 0 and 'qa!' or 'cq!')
