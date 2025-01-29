#!/usr/bin/env lua
local script_path = debug.getinfo(1, "S").source:sub(2)
local script_dir = script_path:match("(.*[/\\])") or "./"

package.path = package.path
	.. ";"
	.. script_dir
	.. "lua_modules/share/lua/5.1/?/init.lua"
	.. ";"
	.. script_dir
	.. "lua_modules/share/lua/5.1/vips/?.lua"

local ok, vips = pcall(require, "vips")
if not ok then
	io.stderr:write("Error: libvips Lua binding required. Install with:\n")
	io.stderr:write("luarocks install lua-vips\n")
	os.exit(1)
end

local os_type

if package.config:sub(1, 1) == "\\" then
	os_type = "win"
else
	os_type = "unix"
end

-- Default settings
local VERSION = "1.1.0"
local DEFAULT_SIZE = "1000x1000"
local DEFAULT_FONT_SIZE = 23
local DEFAULT_BG_COLOR = "#FFFFFF"
local DEFAULT_FG_COLOR = "#000000"
local DEFAULT_BG_ALPHA = 1.0
local DEFAULT_FG_ALPHA = 1.0
local DEFAULT_PREVIEW_TEXT = [[ABCDEFGHIJKLM
NOPQRSTUVWXYZ
abcdefghijklm
nopqrSTUVWXYZ
1234567890
!@$%(){}[]
السلام عليكم]]

local function show_help()
	print(
		([=[
fontpreview.lua %s

Usage: fontpreview.lua -i <input> [options]

Required:
  -i, --input <file>       Input font file (.ttf, .otf, .woff, .woff2)

Options:
  -o, --output <file>      Output image file (default: <input>.png)
  --size <WxH>             Image dimensions (default: %s)
  --font-size <num>        Font size in points (default: %d)
  --bg-color <color>       Background color (default: %s)
  --fg-color <color>       Text color (default: %s)
  --bg-alpha <num>         Background transparency (0.0 - 1.0, default: %.1f)
  --fg-alpha <num>         Text transparency (0.0 - 1.0, default: %.1f)
  --preview-text <text>    Text to render (default: multiline sample)
  -h, --help               Show this help
  --version                Show version

Examples:
  fontpreview.lua -i font.ttf -o preview.png --size 800x600
  fontpreview.lua -i font.otf --fg-color "#FF0000" --bg-color "#000000" --fg-alpha 0.5
]=]):format(
			VERSION,
			DEFAULT_SIZE,
			DEFAULT_FONT_SIZE,
			DEFAULT_BG_COLOR,
			DEFAULT_FG_COLOR,
			DEFAULT_BG_ALPHA,
			DEFAULT_FG_ALPHA
		)
	)
end

local function parse_hex_color(hex)
	hex = hex:gsub("#", "")
	if #hex == 3 then
		local r = tonumber(hex:sub(1, 1), 16) * 17
		local g = tonumber(hex:sub(2, 2), 16) * 17
		local b = tonumber(hex:sub(3, 3), 16) * 17
		return r, g, b
	elseif #hex == 6 then
		local r = tonumber(hex:sub(1, 2), 16)
		local g = tonumber(hex:sub(3, 4), 16)
		local b = tonumber(hex:sub(5, 6), 16)
		return r, g, b
	else
		error("Invalid color format: " .. hex)
	end
end

local function parse_args()
	local config = {
		input = nil,
		output = nil,
		size = DEFAULT_SIZE,
		font_size = DEFAULT_FONT_SIZE,
		bg_color = DEFAULT_BG_COLOR,
		fg_color = DEFAULT_FG_COLOR,
		bg_alpha = DEFAULT_BG_ALPHA,
		fg_alpha = DEFAULT_FG_ALPHA,
		preview_text = DEFAULT_PREVIEW_TEXT,
	}

	local i = 1
	while i <= #arg do
		local a = arg[i]
		if a == "-i" or a == "--input" then
			config.input = arg[i + 1]
			i = i + 1
		elseif a == "-o" or a == "--output" then
			config.output = arg[i + 1]
			i = i + 1
		elseif a == "--size" then
			config.size = arg[i + 1]
			i = i + 1
		elseif a == "--font-size" then
			config.font_size = tonumber(arg[i + 1])
			i = i + 1
		elseif a == "--bg-color" then
			config.bg_color = arg[i + 1]
			i = i + 1
		elseif a == "--fg-color" then
			config.fg_color = arg[i + 1]
			i = i + 1
		elseif a == "--bg-alpha" then
			config.bg_alpha = tonumber(arg[i + 1])
			i = i + 1
		elseif a == "--fg-alpha" then
			config.fg_alpha = tonumber(arg[i + 1])
			i = i + 1
		elseif a == "--preview-text" then
			config.preview_text = arg[i + 1]
			i = i + 1
		elseif a == "-h" or a == "--help" then
			show_help()
			os.exit(0)
		elseif a == "--version" then
			print(VERSION)
			os.exit(0)
		else
			io.stderr:write(("Unknown option: %s\n"):format(a))
			os.exit(1)
		end
		i = i + 1
	end

	if not config.input then
		io.stderr:write("Error: Input font file is required (-i/--input)\n")
		os.exit(1)
	end

	if not config.output then
		config.output = config.input:gsub("%..+$", "") .. ".png"
	end

	return config
end
local function create_srgb_background(width, height, r, g, b, a)
	-- Create background in sRGB color space
	local bg = vips.Image.black(width, height):cast("uchar")
	-- Convert to sRGB color space with alpha channel
	bg = bg:colourspace("srgb"):bandjoin(255)
	-- Fill with specified color
	bg = bg:draw_rect({ r, g, b, math.floor(a * 255) }, 0, 0, width, height, { fill = true })
	return bg
end

local function get_font_name(font_path)
	if os_type == "win" then
		local command = string.format(
			'powershell -Command "[System.Drawing.FontFamily]::Families | Where-Object { $_.GetName(0) -eq ("%s") } | Select-Object -ExpandProperty Name"',
			font_path
		)
		local handle = io.popen(command)
		if handle then
			local result = handle:read("*a")
			handle:close()
			if result ~= "" then
				return result
			else
				print("Font not found")
			end
		end
		if handle then
			local result = handle:read("*a")
			handle:close()
			print(result) -- Prints font names
		end
	elseif os_type == "unix" then
		local command = string.format('fc-scan --format %%{family} "%s"', font_path)
		local handle = io.popen(command)
		if handle then
			local result = handle:read("*a")
			handle:close()
			if result ~= "" then
				-- Match the first result of the response
				return (result:match("([^,]+)"))
			else
				print("Font not found")
			end
		end
	end
	return "NA"
end

local function contains_latin_language_code(codes)
	-- Use hash table for O(1) lookups
	local latin_languages = {
		aa = true,
		af = true,
		ay = true,
		bi = true,
		br = true,
		bs = true,
		ca = true,
		ch = true,
		co = true,
		cs = true,
		cy = true,
		da = true,
		de = true,
		en = true,
		eo = true,
		es = true,
		et = true,
		eu = true,
		fi = true,
		fj = true,
		fo = true,
		fr = true,
		fur = true,
		fy = true,
		gd = true,
		gl = true,
		gv = true,
		ho = true,
		hr = true,
		hu = true,
		ia = true,
		id = true,
		ie = true,
		io = true,
		is = true,
		it = true,
		ki = true,
		kl = true,
		la = true,
		lb = true,
		lt = true,
		lv = true,
		mg = true,
		mh = true,
		mt = true,
		nb = true,
		nds = true,
		nl = true,
		nn = true,
		no = true,
		nr = true,
		nso = true,
		ny = true,
		oc = true,
		om = true,
		pl = true,
		pt = true,
		rm = true,
		ro = true,
		se = true,
		sk = true,
		sl = true,
		sma = true,
		smj = true,
		smn = true,
		so = true,
		sq = true,
		ss = true,
		st = true,
		sv = true,
		sw = true,
		tk = true,
		tl = true,
		tn = true,
		tr = true,
		ts = true,
		uz = true,
		vo = true,
		vot = true,
		wa = true,
		wen = true,
		wo = true,
		xh = true,
		yap = true,
		zu = true,
		an = true,
		crh = true,
		csb = true,
		fil = true,
		hsb = true,
		ht = true,
		jv = true,
		kj = true,
		["ku-tr"] = true,
		kwm = true,
		lg = true,
		li = true,
		ms = true,
		na = true,
		ng = true,
		["pap-an"] = true,
		["pap-aw"] = true,
		rn = true,
		rw = true,
		sc = true,
		sg = true,
		sn = true,
		su = true,
		ty = true,
		za = true,
		agr = true,
		ayc = true,
		bem = true,
		dsb = true,
		lij = true,
		mfe = true,
		mjw = true,
		nhn = true,
		niu = true,
		sgs = true,
		szl = true,
		tpi = true,
		unm = true,
		wae = true,
		yuw = true,
	}

	-- Directly iterate through codes without building a temporary list
	for code in codes:gmatch("([^|]+)") do
		if latin_languages[code] then
			return true
		end
	end
	return false
end

local function generate_preview(config)
	local width, height = config.size:match("^(%d+)x(%d+)$")
	if not width or not height then
		error("Invalid size format. Use WIDTHxHEIGHT (e.g., 800x600)")
	end
	width = tonumber(width)
	height = tonumber(height)

	-- Parse background color and alpha
	local bg_r, bg_g, bg_b = parse_hex_color(config.bg_color)

	-- Create background in sRGB color space
	local bg = create_srgb_background(width, height, bg_r, bg_g, bg_b, config.bg_alpha)

	-- Parse foreground color
	local fg_r, fg_g, fg_b = parse_hex_color(config.fg_color)

	-- Generate text
	local font_path = os.getenv("PWD") .. "/" .. config.input
	local font_family = get_font_name(font_path)
	local ok, text = pcall(function()
		local text_img = vips.Image.text(config.preview_text, {
			font = font_family .. " " .. config.font_size, -- Controls size based on fontconfig
			fontfile = font_path,
			width = width,
			height = height,
			align = "centre",
			dpi = 300,
			rgba = true,
			wrap = true,
		})

		-- Ensure text is in sRGB color space
		if text_img:interpretation() ~= "srgb" then
			text_img = text_img:colourspace("srgb")
		end

		-- Apply text color and alpha
		local alpha = text_img:extract_band(3)
		text_img = text_img:extract_band(0, { n = 3 })
		text_img = text_img:new_from_image({ fg_r, fg_g, fg_b }):bandjoin(alpha * config.fg_alpha)

		return text_img
	end)

	if not ok then
		error("Failed to generate text: " .. tostring(text))
	end

	local command = string.format("fc-scan --format %%{lang} %s", font_path)
	local handle = io.popen(command)
	local lang_codes
	if handle then
		lang_codes = handle:read("*a")
		handle:close()
	end

	if not contains_latin_language_code(lang_codes) then
		print("Font has not declared support for a latin script, expect the unexpected.")
	end

	-- Ensure both images have the same format before compositing
	bg = bg:cast(text:format())

	local x = math.max(0, math.floor((bg:width() - text:width()) / 2))
	local y = math.max(0, math.floor((bg:height() - text:height()) / 2))

	-- Composite and save
	local output = bg:composite2(text, "over", { x = x, y = x })
	output:write_to_file(config.output)
	print(("Generated preview: %s"):format(config.output))
end

-- Main execution
local ok, err = pcall(function()
	local config = parse_args()
	generate_preview(config)
end)

if not ok then
	io.stderr:write(("Error: %s\n"):format(err))
	os.exit(1)
end
