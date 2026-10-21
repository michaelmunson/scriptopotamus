-- Lightweight, line-based scan of a scrippo buffer. It is intentionally
-- forgiving: completion has to work on half-typed code.
local M = {}

local KEYWORDS = { ['if'] = true, elif = true, ['else'] = true, ['for'] = true, ['while'] = true, case = true }

local function indent_of(line)
  return #line:match('^[ \t]*')
end

local function is_blank(line)
  return line:match('^%s*$') ~= nil
end

--- Remove a trailing `#` comment that is outside of quotes.
function M.strip_comment(line)
  local quote
  for i = 1, #line do
    local c = line:sub(i, i)
    if quote then
      if c == quote then
        quote = nil
      end
    elseif c == '"' or c == "'" then
      quote = c
    elseif c == '#' and (i == 1 or line:sub(i - 1, i - 1):match('[%s;]')) then
      return line:sub(1, i - 1)
    end
  end
  return line
end

local function var_name(flag_or_name)
  return (flag_or_name:gsub('^%-+', ''):gsub('%-', '_'))
end

--- Parse the text between a header's parentheses into parameter leaves.
function M.parse_params(inner)
  local text = inner:gsub('<[^<>]*>', ' '):gsub('%.%.%.', ' '):gsub('[%[%]|]', ' '):gsub('/', ' / ')
  local leaves = {}
  local alias = false
  local last_opt
  for tok in text:gmatch('%S+') do
    if tok == '/' then
      alias = true
    elseif tok:sub(1, 1) == '-' and tok:match('^%-%-?[%w_][%w_-]*$') then
      if alias and last_opt then
        table.insert(last_opt.flags, tok)
      else
        last_opt = { kind = 'opt', flags = { tok } }
        table.insert(leaves, last_opt)
      end
      alias = false
    elseif tok:match('^%*[%a_][%w_-]*$') then
      table.insert(leaves, { kind = 'arg', name = tok:sub(2), rest = true })
      alias = false
      last_opt = nil
    elseif tok:match('^[%a_][%w_-]*$') then
      table.insert(leaves, { kind = 'arg', name = tok })
      alias = false
      last_opt = nil
    else
      alias = false
    end
  end
  for _, leaf in ipairs(leaves) do
    if leaf.kind == 'opt' then
      local long
      for _, flag in ipairs(leaf.flags) do
        if flag:sub(1, 2) == '--' then
          long = long or flag
        end
      end
      leaf.var = var_name(long or leaf.flags[1])
    else
      leaf.var = leaf.name:gsub('%-', '_')
    end
  end
  return leaves
end

function M.describe_param(leaf)
  if leaf.kind == 'opt' then
    return table.concat(leaf.flags, ' / ')
  end
  return (leaf.rest and '*' or '') .. leaf.name
end

local function parse_type(text)
  local words = {}
  for w in text:gmatch('[%w_]+') do
    words[#words + 1] = w
  end
  local is_const = false
  local base
  for _, w in ipairs(words) do
    if w == 'const' then
      is_const = true
    elseif not base then
      base = w
    end
  end
  return { is_const = is_const, base = base or (is_const and 'any' or nil), text = vim.trim((text:gsub('=.*$', ''))) }
end

local function collect_header(lines, start, rest)
  local depth = 0
  local quote
  local parts = {}
  local i = start
  local text = rest
  while true do
    local clean = M.strip_comment(text)
    local piece = {}
    for pos = 1, #clean do
      local c = clean:sub(pos, pos)
      if quote then
        piece[#piece + 1] = c
        if c == quote then quote = nil end
      elseif c == '"' or c == "'" then
        quote = c
        piece[#piece + 1] = c
      elseif c == '(' then
        depth = depth + 1
        if depth > 1 then
          piece[#piece + 1] = c
        end
      elseif c == ')' then
        depth = depth - 1
        if depth == 0 then
          parts[#parts + 1] = table.concat(piece)
          return table.concat(parts, ' '), i
        end
        piece[#piece + 1] = c
      elseif depth >= 1 then
        piece[#piece + 1] = c
      end
    end
    parts[#parts + 1] = table.concat(piece)
    i = i + 1
    if i > #lines then
      return table.concat(parts, ' '), #lines, true
    end
    text = lines[i]
  end
end

--- Scan buffer lines (1-based rows). Returns the analysis table.
function M.analyze(lines)
  local result = {
    imports = {},
    macros = {},
    defs = {},
    variables = {},
  }

  local function add_var(name, line, typ)
    if not result.variables[name] then
      result.variables[name] = { name = name, line = line, type = typ }
    end
  end

  local i = 1
  while i <= #lines do
    local line = lines[i]
    local code = M.strip_comment(line)
    local ws, dot, name, rest = code:match('^([ \t]*)(%.?)([%a_][%w_-]*)(%(.*)$')
    if not ws then
      ws, dot, rest = code:match('^([ \t]*)(%.)(%(.*)$')
      name = ''
    end
    local macro_ws, macro_name, macro_rest = code:match('^([ \t]*)@([%a_][%w_]*)(%(.*)$')

    if macro_name then
      local inner, close = collect_header(lines, i, line:sub(#macro_ws + #macro_name + 2))
      if macro_name ~= 'info' then
        table.insert(result.macros, { name = macro_name, line = i, close = close, header = inner })
      end
      i = close
    elseif ws and not (dot == '' and KEYWORDS[name]) then
      local inner, close = collect_header(lines, i, line:sub(#ws + #dot + #name + 1))
      local def = {
        name = name,
        kind = dot == '.' and 'command' or 'function',
        line = i,
        close = close,
        indent = #ws,
        leaves = M.parse_params(inner),
        header = (dot .. name .. '(' .. vim.trim(inner) .. ')'),
      }
      table.insert(result.defs, def)
      i = close
    else
      local import = code:match('^%s*@import%s+(.+)$')
      if import then
        table.insert(result.imports, import)
      end
      local decl, type_text = code:match('^%s*([%a_][%w_]*)<([^<>]*)>')
      if decl then
        add_var(decl, i, parse_type(type_text))
      else
        local assigned = code:match('^%s*([%a_][%w_]*)=')
        if assigned then
          add_var(assigned, i)
        end
      end
      local loop_var = code:match('^%s*for%s+([%a_][%w_]*)%s+in%f[%s%z]')
      if loop_var then
        add_var(loop_var, i)
      end
      for target in code:gmatch('%s%-%-?v%s+([%a_][%w_]*)') do
        add_var(target, i)
      end
    end
    i = i + 1
  end

  for _, def in ipairs(result.defs) do
    local last = def.close
    for k = def.close + 1, #lines do
      if not is_blank(lines[k]) then
        if indent_of(lines[k]) <= def.indent then
          break
        end
        last = k
      end
    end
    def.last = last
  end

  local stack = {}
  for _, def in ipairs(result.defs) do
    while #stack > 0 and stack[#stack].last < def.line do
      table.remove(stack)
    end
    def.parent = stack[#stack]
    stack[#stack + 1] = def
  end

  return result
end

--- Definitions whose header or body contains `row`, outermost first.
function M.chain_at(analysis, row)
  local chain = {}
  for _, def in ipairs(analysis.defs) do
    if def.line <= row and row <= def.last then
      chain[#chain + 1] = def
    end
  end
  return chain
end

--- Walk `words` through nested commands. Returns node and how many words matched.
function M.resolve_command_path(analysis, words)
  local siblings = {}
  for _, def in ipairs(analysis.defs) do
    if def.kind == 'command' and def.name ~= '' and not def.parent then
      siblings[#siblings + 1] = def
    end
  end
  local node
  local depth = 0
  for _, word in ipairs(words) do
    local found
    for _, candidate in ipairs(siblings) do
      if candidate.name == word then
        found = candidate
        break
      end
    end
    if not found then
      break
    end
    node = found
    depth = depth + 1
    siblings = {}
    for _, def in ipairs(analysis.defs) do
      if def.parent == node and def.kind == 'command' and def.name ~= '' then
        siblings[#siblings + 1] = def
      end
    end
  end
  if node then
    return node, depth
  end
  return nil
end

function M.children(analysis, node)
  local out = {}
  for _, def in ipairs(analysis.defs) do
    if def.parent == node and def.kind == 'command' and def.name ~= '' then
      out[#out + 1] = def
    end
  end
  return out
end

return M
