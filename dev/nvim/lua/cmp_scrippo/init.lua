local source = {}

-- vim.lsp.protocol.CompletionItemKind values
local KINDS = {
  command = 3,
  ['function'] = 3,
  macro = 15,
  keyword = 14,
  param = 6,
  variable = 6,
  constant = 21,
  flag = 12,
  type = 25,
  module = 9,
}

function source.new()
  return setmetatable({}, { __index = source })
end

function source:get_debug_name()
  return 'scrippo'
end

function source:is_available()
  return vim.bo.filetype == 'scrippo'
end

function source:get_trigger_characters()
  return { '@', '$', '-', '.', '<', '/', '{' }
end

function source:complete(params, callback)
  local complete = require('scrippo.complete')
  local cursor = params.context.cursor
  local before = params.context.cursor_before_line
  local buf = params.context.bufnr or vim.api.nvim_get_current_buf()
  local result = complete.complete_buffer(buf, cursor.row, #before)
  if not result then
    callback({})
    return
  end
  local items = {}
  for _, item in ipairs(result.items) do
    local new_text = item.snippet or item.insert or item.label
    items[#items + 1] = {
      label = item.label,
      kind = KINDS[item.kind],
      detail = item.detail,
      documentation = item.doc and { kind = 'markdown', value = item.doc } or nil,
      filterText = item.match_text or item.label,
      sortText = item.sort,
      insertTextFormat = item.snippet and 2 or 1,
      textEdit = {
        newText = new_text,
        range = {
          start = { line = cursor.row - 1, character = result.start },
          ['end'] = { line = cursor.row - 1, character = #before },
        },
      },
    }
  end
  callback({ items = items, isIncomplete = false })
end

return source
