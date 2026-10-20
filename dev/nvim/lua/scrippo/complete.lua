local analyzer = require('scrippo.analyzer')
local data = require('scrippo.data')

local M = {}

local SKIP_DIRS = { ['.git'] = true, node_modules = true, target = true, dist = true, ['.scrippo'] = true }

local function codeblock(code)
  return '```scriptopotamus\n' .. code .. '\n```'
end

local function starts_with(str, prefix)
  return str:sub(1, #prefix) == prefix
end

--- Blank out quoted text so separators inside strings are ignored.
--- Returns the masked text and the quote character left open, if any.
local function mask_quotes(text)
  local out = {}
  local quote
  for i = 1, #text do
    local c = text:sub(i, i)
    if quote then
      out[i] = (c == quote) and c or ' '
      if c == quote then
        quote = nil
      end
    else
      out[i] = c
      if c == '"' or c == "'" then
        quote = c
      end
    end
  end
  return table.concat(out), quote
end

local function imported_modules(analysis)
  local modules = {}
  for _, raw in ipairs(analysis.imports) do
    for _, name in ipairs(data.modules_for_import(raw)) do
      modules[name] = true
    end
  end
  return modules
end

local function macro_items(analysis, with_at)
  local items = {}
  local seen = {}
  local names = {}
  for name, builtin in pairs(data.BUILTINS) do
    if builtin.macro then
      names[#names + 1] = name
    end
  end
  table.sort(names)
  for _, name in ipairs(names) do
    local builtin = data.BUILTINS[name]
    seen[name] = true
    items[#items + 1] = {
      label = with_at and ('@' .. name) or name,
      kind = 'macro',
      detail = '@' .. name,
      doc = codeblock(builtin.signature) .. '\n\n' .. builtin.description,
      insert = with_at and ('@' .. name) or name,
      snippet = with_at and ('@' .. builtin.snippet) or builtin.snippet,
      match_text = name,
    }
  end
  for _, macro in ipairs(analysis.macros) do
    if not seen[macro.name] then
      seen[macro.name] = true
      items[#items + 1] = {
        label = with_at and ('@' .. macro.name) or macro.name,
        kind = 'macro',
        detail = '@' .. macro.name,
        doc = codeblock('@' .. macro.name .. '()') .. '\n\nMacro defined on line ' .. macro.line,
        insert = with_at and ('@' .. macro.name) or macro.name,
        match_text = macro.name,
      }
    end
  end
  return items
end

local function callable_item(def)
  local parts = {}
  for _, leaf in ipairs(def.leaves) do
    parts[#parts + 1] = analyzer.describe_param(leaf)
  end
  return {
    label = def.name,
    kind = def.kind,
    detail = table.concat(parts, ' '),
    doc = codeblock(def.header) .. '\n\n' .. (def.kind == 'command' and 'Command' or 'Function') .. ' defined on line ' .. def.line,
  }
end

local function function_items(analysis)
  local items = {}
  local defined = {}
  for _, def in ipairs(analysis.defs) do
    if def.name ~= '' and (def.kind == 'function' or not def.parent) then
      defined[def.name] = true
      items[#items + 1] = callable_item(def)
    end
  end
  local imported = imported_modules(analysis)
  local names = {}
  for name, builtin in pairs(data.BUILTINS) do
    if not builtin.macro and imported[builtin.pkg] and not defined[name] then
      names[#names + 1] = name
    end
  end
  table.sort(names)
  for _, name in ipairs(names) do
    local builtin = data.BUILTINS[name]
    items[#items + 1] = {
      label = name,
      kind = 'function',
      detail = 'scrippo/std/' .. builtin.pkg,
      doc = codeblock(builtin.signature) .. '\n\n' .. builtin.description,
      snippet = builtin.snippet,
    }
  end
  return items
end

local function keyword_items()
  local items = {}
  for _, name in ipairs(data.KEYWORD_ORDER) do
    items[#items + 1] = {
      label = name,
      kind = 'keyword',
      doc = data.KEYWORDS[name],
      snippet = data.KEYWORD_SNIPPETS[name],
    }
  end
  return items
end

local function variable_items(analysis, row)
  local items = {}
  local seen = {}
  for _, def in ipairs(analyzer.chain_at(analysis, row)) do
    for _, leaf in ipairs(def.leaves) do
      if not seen[leaf.var] then
        seen[leaf.var] = true
        items[#items + 1] = {
          label = leaf.var,
          kind = 'param',
          detail = analyzer.describe_param(leaf),
          doc = (leaf.kind == 'opt' and 'Option' or 'Argument') .. ' of `' .. (def.name ~= '' and def.name or '.()') .. '`',
          sort = '0' .. leaf.var,
        }
      end
    end
  end
  local names = vim.tbl_keys(analysis.variables)
  table.sort(names)
  for _, name in ipairs(names) do
    if not seen[name] then
      local variable = analysis.variables[name]
      local is_const = variable.type and variable.type.is_const
      items[#items + 1] = {
        label = name,
        kind = is_const and 'constant' or 'variable',
        detail = variable.type and ('<' .. variable.type.text .. '>') or '',
        doc = (is_const and 'Constant' or 'Variable') .. ' declared on line ' .. variable.line,
        sort = '1' .. name,
      }
    end
  end
  return items
end

local function option_ref_items(analysis, row)
  local items = {}
  local seen = {}
  for _, def in ipairs(analyzer.chain_at(analysis, row)) do
    for _, leaf in ipairs(def.leaves) do
      if leaf.kind == 'opt' then
        for _, flag in ipairs(leaf.flags) do
          if not seen[flag] then
            seen[flag] = true
            items[#items + 1] = {
              label = flag,
              kind = 'flag',
              detail = analyzer.describe_param(leaf),
              doc = 'Option of `' .. (def.name ~= '' and def.name or '.()') .. '`, also available as `$' .. leaf.var .. '`',
            }
          end
        end
      end
    end
  end
  return items
end

local function type_items(previous_words)
  local has_const = false
  for _, w in ipairs(previous_words) do
    if w ~= 'const' then
      return {}
    end
    has_const = true
  end
  local names = {}
  if not has_const then
    names[#names + 1] = 'const'
  end
  vim.list_extend(names, data.TYPE_NAMES)
  local items = {}
  for _, name in ipairs(names) do
    items[#items + 1] = { label = name, kind = 'type', doc = data.TYPES[name] }
  end
  return items
end

local function flag_items_for(def, used)
  local items = {}
  local function is_used(flag)
    return vim.tbl_contains(used, flag)
  end
  for _, leaf in ipairs(def.leaves) do
    if leaf.kind == 'opt' then
      local taken = false
      for _, flag in ipairs(leaf.flags) do
        taken = taken or is_used(flag)
      end
      if not taken then
        for _, flag in ipairs(leaf.flags) do
          items[#items + 1] = {
            label = flag,
            kind = 'flag',
            detail = analyzer.describe_param(leaf),
            doc = 'Option of `' .. def.name .. '`',
          }
        end
      end
    end
  end
  return items
end

local function flag_items(analysis, command, used)
  local node = analyzer.resolve_command_path(analysis, { command })
  if not node then
    for _, def in ipairs(analysis.defs) do
      if def.kind == 'function' and def.name == command then
        node = def
      end
    end
  end
  if node then
    return flag_items_for(node, used)
  end
  local builtin = data.BUILTINS[command]
  if not builtin or builtin.macro or not imported_modules(analysis)[builtin.pkg] then
    return {}
  end
  local items = {}
  for _, flag in ipairs(builtin.flags) do
    if not vim.tbl_contains(used, flag) then
      items[#items + 1] = { label = flag, kind = 'flag', detail = command }
    end
  end
  return items
end

local function scrippo_home()
  local home = vim.env.SCRIPPO_HOME
  if home and home ~= '' then
    return home
  end
  return vim.fn.expand('~') .. '/.scrippo'
end

local function walk_scrippo_files(root)
  local out = {}
  local ok, iter = pcall(vim.fs.dir, root, {
    depth = 8,
    -- vim.fs.dir descends into a directory only while `skip` returns true.
    skip = function(name)
      return not (SKIP_DIRS[name] or name:sub(1, 1) == '.')
    end,
  })
  if not ok then
    return out
  end
  for name, kind in iter do
    if kind == 'file' and name:sub(-8) == '.scrippo' and not vim.fs.basename(name):match('^%.') then
      out[#out + 1] = './' .. name
    end
  end
  table.sort(out)
  return out
end

local function library_names(dir)
  local out = {}
  for _, file in ipairs(vim.fn.glob(dir .. '/*.scrippo', false, true)) do
    out[#out + 1] = vim.fn.fnamemodify(file, ':t:r')
  end
  table.sort(out)
  return out
end

local function import_items(typed, quote, import_dir)
  local items = {}
  local show_packages = typed == '' or starts_with(typed, 'scrippo') or starts_with('scrippo', typed)
  if show_packages then
    for index, pkg in ipairs(data.PACKAGES) do
      items[#items + 1] = {
        label = pkg.path,
        kind = 'module',
        detail = pkg.detail,
        doc = pkg.description,
        insert = pkg.path .. quote,
        sort = string.format('0%02d', index),
      }
    end
  end

  if import_dir and typed:match('^[%w_-]*$') then
    local project = vim.fs.find('.scrippo', { upward = true, type = 'directory', path = import_dir })[1]
    local libs = {}
    if project then
      libs[#libs + 1] = { dir = project .. '/lib', detail = 'project package' }
    end
    libs[#libs + 1] = { dir = scrippo_home() .. '/lib', detail = 'global package' }
    local seen = {}
    for _, lib in ipairs(libs) do
      for _, name in ipairs(library_names(lib.dir)) do
        if not seen[name] then
          seen[name] = true
          items[#items + 1] = {
            label = name,
            kind = 'module',
            detail = lib.detail,
            doc = 'Import package `' .. name .. '` from `' .. lib.dir .. '`.',
            insert = name .. quote,
            sort = '1' .. name,
          }
        end
      end
    end
  end

  if import_dir then
    for index, file in ipairs(walk_scrippo_files(import_dir)) do
      local candidate = file
      local matches = typed == ''
        or starts_with(candidate, typed)
        or starts_with(candidate:sub(3), typed)
        or candidate:find('/' .. typed, 1, true) ~= nil
      if matches then
        items[#items + 1] = {
          label = file,
          kind = 'module',
          detail = 'local file',
          doc = 'Import `' .. file .. '` relative to this script.',
          insert = file .. quote,
          match_text = typed,
          sort = string.format('2%04d', index),
        }
      end
    end
  end
  return items
end

--- Split what follows `@import` into its path token and the rest of the line.
local function split_import_args(text)
  local quote = text:sub(1, 1)
  if quote == '"' or quote == "'" then
    local close = text:find(quote, 2, true)
    if not close then
      return nil
    end
    return text:sub(1, close), text:sub(close + 1)
  end
  local token, rest = text:match('^(%S+)(%s.*)$')
  if not token then
    return nil
  end
  return token, rest
end

local function import_completion(prefix, import_dir)
  local args = prefix:match('^%s*@import%s+(.*)$')
  if not args then
    return nil
  end
  local token, rest = split_import_args(args)
  if token then
    local scope = rest:match('^%s+%-%-as%s+(%.?)$')
    if scope then
      return {
        start = #prefix - #scope,
        items = {
          { label = '.command', kind = 'command', insert = '.', snippet = '.${1:command}', match_text = '.' },
          { label = 'function', kind = 'function', insert = '', snippet = '${1:function}' },
        },
      }
    end
    if rest:match('^%s+%-%-as%s+') then
      return nil
    end
    local flag = rest:match('^%s+(%-*[%w-]*)$')
    if flag then
      return {
        start = #prefix - #flag,
        items = {
          {
            label = '--as',
            kind = 'flag',
            detail = 'Import into a command or function scope',
            insert = '--as ',
            snippet = '--as ${1:.command}',
          },
        },
      }
    end
    return nil
  end
  local quote, typed = args:match('^(["\']?)([^%s"\']*)$')
  if not quote then
    return nil
  end
  local items = import_items(typed, quote, import_dir)
  if #items == 0 then
    return nil
  end
  return { start = #prefix - #typed, items = items }
end

local function in_header(analysis, lines, row, col)
  for _, def in ipairs(analysis.defs) do
    if def.line <= row and row <= def.close then
      local depth = 0
      local started = false
      for lnum = def.line, row do
        local text = lines[lnum] or ''
        if lnum == row then
          text = text:sub(1, col)
        end
        text = analyzer.strip_comment(text)
        if lnum == def.line then
          text = text:sub(def.indent + 1)
        end
        for pos = 1, #text do
          local c = text:sub(pos, pos)
          if c == '(' then
            depth = depth + 1
            started = true
          elseif c == ')' then
            depth = depth - 1
          end
        end
      end
      if started and depth > 0 then
        return true
      end
    end
  end
  return false
end

local function type_completion(prefix, header)
  local before, inside = prefix:match('^(.*[%w_])<([^<>]*)$')
  if not before then
    return nil
  end
  local is_declaration = before:match('^%s*[%a_][%w_]*$') ~= nil
  if not (is_declaration or header) or inside:find('[={(]') then
    return nil
  end
  local words = {}
  for w in inside:gmatch('%S+') do
    words[#words + 1] = w
  end
  local current = ''
  if not inside:match('%s$') and #words > 0 then
    current = table.remove(words)
  end
  return { start = #prefix - #current, items = type_items(words) }
end

local function command_completion(analysis, prefix, masked)
  local cut = 0
  for pos = 1, #masked do
    if masked:sub(pos, pos):match('[;&|(`{]') then
      cut = pos
    end
  end
  local segment = prefix:sub(cut + 1):gsub('^%s+', '')
  local lead = ''
  local after_if = segment:match('^[%a]+%s+')
  if after_if and (after_if:match('^if%s') or after_if:match('^elif%s') or after_if:match('^while%s')) then
    lead = after_if
  end
  local negation = segment:sub(#lead + 1):match('^!%s+')
  if negation then
    lead = lead .. negation
  end
  local rest = segment:sub(#lead + 1)
  local words = {}
  for w in rest:gmatch('%S+') do
    words[#words + 1] = w
  end
  local current = ''
  if not rest:match('%s$') and #words > 0 then
    current = table.remove(words)
  end
  local start = #prefix - #current

  if #words == 0 then
    if current ~= '' and not current:match('^[%a_][%w_-]*$') then
      return nil
    end
    local items = function_items(analysis)
    vim.list_extend(items, macro_items(analysis, true))
    if cut == 0 and lead == '' then
      vim.list_extend(items, keyword_items())
    end
    return { start = start, items = items }
  end

  local node, depth = analyzer.resolve_command_path(analysis, words)
  if node then
    local used = vim.list_slice(words, depth + 1)
    if current:sub(1, 1) == '-' then
      return { start = start, items = flag_items_for(node, used) }
    end
    if depth == #words then
      local children = analyzer.children(analysis, node)
      if #children > 0 then
        local items = {}
        for _, child in ipairs(children) do
          items[#items + 1] = callable_item(child)
        end
        return { start = start, items = items }
      end
    end
    return nil
  end

  if current:sub(1, 1) ~= '-' then
    return nil
  end
  return { start = start, items = flag_items(analysis, words[1], vim.list_slice(words, 2)) }
end

--- Compute completions at a position.
--- `lines` is the full buffer, `row` is 1-based, `col` is the 0-based byte offset of the cursor.
--- Returns `{ start = <0-based byte col>, items = {...} }` or nil.
function M.complete(lines, row, col, opts)
  opts = opts or {}
  local line = lines[row] or ''
  local prefix = line:sub(1, col)
  local masked, open_quote = mask_quotes(prefix)
  if analyzer.strip_comment(prefix) ~= prefix or open_quote == "'" then
    return nil
  end
  local analysis = analyzer.analyze(lines)

  local at_typed = prefix:match('@([%w_]*)$')
  if at_typed then
    local before = prefix:sub(-#at_typed - 2, -#at_typed - 2)
    if before == '' or not before:match('[%w_$]') then
      return { start = #prefix - #at_typed, items = macro_items(analysis, false) }
    end
  end

  local option_typed = prefix:match('%$(%-%-?[%w_-]*)$')
  if option_typed then
    return { start = #prefix - #option_typed, items = option_ref_items(analysis, row) }
  end

  local variable_typed = prefix:match('%$%{?([%w_]*)$')
  if variable_typed then
    return { start = #prefix - #variable_typed, items = variable_items(analysis, row) }
  end

  local imported = import_completion(prefix, opts.import_dir)
  if imported or open_quote == '"' then
    return imported
  end

  local header = in_header(analysis, lines, row, col)
  if header then
    return type_completion(prefix, true)
  end
  return type_completion(prefix, false)
    or command_completion(analysis, prefix, masked)
end

local KIND_LABELS = {
  command = 'command',
  ['function'] = 'function',
  macro = 'macro',
  keyword = 'keyword',
  param = 'param',
  variable = 'variable',
  constant = 'constant',
  flag = 'flag',
  type = 'type',
  module = 'module',
}

local function sort_items(items)
  table.sort(items, function(a, b)
    local ka = a.sort or ('5' .. a.label)
    local kb = b.sort or ('5' .. b.label)
    if ka ~= kb then
      return ka < kb
    end
    return a.label < b.label
  end)
  return items
end

local function buffer_dir(buf)
  local name = vim.api.nvim_buf_get_name(buf)
  if name == '' then
    return vim.fn.getcwd()
  end
  return vim.fn.fnamemodify(name, ':p:h')
end

--- Completion for the buffer at the cursor, as `vim.fn.complete()`-style items.
function M.complete_buffer(buf, row, col)
  local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
  return M.complete(lines, row, col, { import_dir = buffer_dir(buf) })
end

function M.filter_items(items, base)
  local lowered = base:lower()
  local out = {}
  for _, item in ipairs(items) do
    local text = (item.match_text or item.label):lower()
    if starts_with(text, lowered) then
      out[#out + 1] = item
    end
  end
  return sort_items(out)
end

function M.to_vim_items(items)
  local out = {}
  for _, item in ipairs(items) do
    out[#out + 1] = {
      word = item.insert or item.label,
      abbr = item.label,
      kind = KIND_LABELS[item.kind] or item.kind,
      menu = item.detail or '',
      info = item.doc and item.doc:gsub('```%w*\n?', '') or '',
      icase = 1,
      dup = 0,
      empty = 1,
    }
  end
  return out
end

local pending

function M.omnifunc(findstart, base)
  if findstart == 1 then
    local pos = vim.api.nvim_win_get_cursor(0)
    pending = M.complete_buffer(0, pos[1], pos[2])
    if not pending then
      return -3
    end
    return pending.start
  end
  if not pending then
    return {}
  end
  local items = M.filter_items(pending.items, base)
  pending = nil
  return M.to_vim_items(items)
end

return M
