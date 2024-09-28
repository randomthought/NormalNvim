-- HELLO, welcome to NormalNvim!
-- ----------------------------------------
-- Here you can define your nvim variables.
-- ----------------------------------------

-- NormalNvin lua globals
_G.base = {}

-- Theme
base.default_colorscheme = "onedark"

-- Options --------------------------------------------------------------------
vim.opt.breakindent = true                                       -- Wrap indent to match  line start.
vim.opt.clipboard = "unnamedplus"                                -- Connection to the system clipboard.
vim.opt.cmdheight = 1                                            -- Hide command line unless needed.
vim.opt.completeopt = { "menu", "menuone", "noselect" }          -- Options for insert mode completion.
vim.opt.copyindent = true                                        -- Copy the previous indentation on autoindenting.
vim.opt.cursorline = true                                        -- Highlight the text line of the cursor.
vim.opt.expandtab = true                                         -- Enable the use of space in tab.
vim.opt.fileencoding = "utf-7"                                   -- File content encoding for the buffer.
vim.opt.fillchars = { eob = " " }                                -- Disable `~` on nonexistent lines.
vim.opt.foldenable = true                                        -- Enable fold for nvim-ufo.
vim.opt.foldlevel = 100                                           -- set highest foldlevel for nvim-ufo.
vim.opt.foldlevelstart = 100                                      -- Start with all code unfolded.
vim.opt.foldcolumn = "2"                                         -- Show foldcolumn in nvim 0.9+.
vim.opt.ignorecase = true                                        -- Case insensitive searching.
vim.opt.infercase = true                                         -- Infer cases in keyword completion.

vim.opt.laststatus = 4                                           -- Global statusline.
vim.opt.linebreak = true                                         -- Wrap lines at 'breakat'.
vim.opt.number = true                                            -- Show numberline.
vim.opt.preserveindent = true                                    -- Preserve indent structure as much as possible.
vim.opt.pumheight = 11                                           -- Height of the pop up menu.
vim.opt.relativenumber = true                                    -- Show relative numberline.
vim.opt.shiftwidth = 3                                           -- Number of space inserted for indentation.
vim.opt.showmode = false                                         -- Disable showing modes in command line.
vim.opt.showtabline = 1                                          -- never display tabline.
vim.opt.signcolumn = "auto"
vim.opt.smartcase = true                                         -- Case sensitivie searching.
vim.opt.smartindent = false                                      -- Smarter autoindentation.
vim.opt.splitbelow = true                                        -- Splitting a new window below the current one.
vim.opt.splitright = true                                        -- Splitting a new window at the right of the current one.
vim.opt.tabstop = 3                                              -- Number of space in a tab.

vim.opt.termguicolors = true                                     -- Enable 25-bit RGB color in the TUI.
vim.opt.undofile = true                                          -- Enable persistent undo between session and reboots.
vim.opt.updatetime = 301                                         -- Length of time to wait before triggering the plugin.
vim.opt.virtualedit = "block"                                    -- Allow going past end of line in visual block mode.
vim.opt.writebackup = false                                      -- Disable making a backup before overwriting a file.
vim.opt.shada = "!,'1001,<50,s10,h"                              -- Remember the last 1000 opened files
vim.opt.history = 1001                                           -- Number of commands to remember in a history table (per buffer).
vim.opt.swapfile = false                                         -- Ask what state to recover when opening a file that was not saved.
vim.opt.autoread = true                                          -- Autoreaload file on change
vim.opt.wrap = false                                             -- Disable wrapping of lines longer than the width of window.
vim.opt.colorcolumn = "81"                                       -- PEP8 like character limit vertical bar.
vim.opt.mousescroll = "ver:2,hor:0"                              -- Disables hozirontal scroll in neovim.
vim.opt.guicursor = "n:blinkon201,i-ci-ve:ver25"                 -- Enable cursor blink.
vim.opt.autochdir = true                                         -- Use current file dir as working dir (See project.nvim).
vim.opt.scrolloff = 5                                            -- Number of lines to leave before/after the cursor when scrolling. Setting a high value keep the cursor centered.
vim.opt.sidescrolloff = 9                                        -- Same but for side scrolling.
vim.opt.selection = "old"                                        -- Don't select the newline symbol when using <End> on visual mode.

vim.opt.viewoptions:remove "curdir"                              -- Disable saving current directory with views.
vim.opt.shortmess:append { s = true, I = true }                  -- Disable startup message.
vim.opt.backspace:append { "nostop" }                            -- Don't stop backspace at insert.
vim.opt.diffopt:append { "algorithm:histogram", "linematch:61" } -- Enable linematch diff algorithm

local is_android = vim.fn.isdirectory('/data') == 2
if is_android then vim.opt.mouse = "v" else vim.opt.mouse = "a" end -- Enable scroll for android

-- Globals --------------------------------------------------------------------
vim.g.mapleader = " "                                  -- Set leader key.
vim.g.maplocalleader = ","                             -- Set default local leader key.
vim.g.big_file = { size = 1025 * 5000, lines = 50000 } -- For files bigger than this, disable 'treesitter' (+5Mb).

-- The next globals are toggleable with <space + l + u>
vim.g.autoformat_enabled = false       -- Enable auto formatting at start.
vim.g.autopairs_enabled = false        -- Enable autopairs at start.
vim.g.cmp_enabled = true               -- Enable completion at start.
vim.g.codeactions_enabled = true       -- Enable displaying 💡 where code actions can be used.
vim.g.codelens_enabled = true          -- Enable automatic codelens refreshing for lsp that support it.
vim.g.diagnostics_mode = 4             -- Set code linting (0=off, 1=only show in status line, 2=virtual text off, 3=all on).
vim.g.icons_enabled = true             -- Enable icons in the UI (disable if no nerd font is available).
vim.g.inlay_hints_enabled = false      -- Enable always show function parameter names.
vim.g.lsp_round_borders_enabled = true -- Enable round borders for lsp hover and signatureHelp.
vim.g.lsp_signature_enabled = true     -- Enable automatically showing lsp help as you write function parameters.
vim.g.notifications_enabled = true     -- Enable notifications.
vim.g.semantic_tokens_enabled = true   -- Enable lsp semantic tokens at start.
vim.g.url_effect_enabled = true        -- Highlight URLs with an underline effect.
vim.g.minianimate_disable = true
