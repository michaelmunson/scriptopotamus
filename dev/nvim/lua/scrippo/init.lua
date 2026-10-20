local M = {}

local registered_cmp = false
local attached = {}

local defaults = {
  autotrigger = false,
  min_chars = 1,
}

local config = vim.deepcopy(defaults)

local function register_cmp()
  if registered_cmp then
    return
  end
  local cmp = package.loaded['cmp']
  if not cmp then
    return
  end
  cmp.register_source('scrippo', require('cmp_scrippo').new())
  registered_cmp = true
end

local function autotrigger(buf)
  local group = vim.api.nvim_create_augroup('scrippo_autotrigger_' .. buf, { clear = true })
  vim.api.nvim_create_autocmd('TextChangedI', {
    group = group,
    buffer = buf,
    callback = function()
      if vim.fn.pumvisible() == 1 then
        return
      end
      local pos = vim.api.nvim_win_get_cursor(0)
      local before = vim.api.nvim_get_current_line():sub(1, pos[2])
      if not before:match('[@$%.%-<]$') and not before:match('[%w_]' .. string.rep('[%w_-]', config.min_chars - 1) .. '$') then
        return
      end
      local complete = require('scrippo.complete')
      local result = complete.complete_buffer(buf, pos[1], pos[2])
      if not result then
        return
      end
      local base = before:sub(result.start + 1)
      local items = complete.filter_items(result.items, base)
      if #items == 0 then
        return
      end
      vim.fn.complete(result.start + 1, complete.to_vim_items(items))
    end,
  })
end

--- Optional configuration. Call before opening a scrippo file.
---@param opts? { autotrigger?: boolean, min_chars?: integer }
function M.setup(opts)
  config = vim.tbl_extend('force', config, opts or {})
  register_cmp()
end

--- Called by the ftplugin for every scrippo buffer.
function M.attach(buf)
  register_cmp()
  local wants_autotrigger = vim.g.scrippo_autotrigger
  if wants_autotrigger == nil then
    wants_autotrigger = config.autotrigger
  end
  if wants_autotrigger and wants_autotrigger ~= 0 and not attached[buf] then
    attached[buf] = true
    autotrigger(buf)
    vim.api.nvim_create_autocmd('BufWipeout', {
      buffer = buf,
      once = true,
      callback = function()
        attached[buf] = nil
      end,
    })
  end
end

return M
