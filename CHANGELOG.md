# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] – 2025-05-13

### Added
- Initial Rust rewrite of the font-preview tool as `sia`.  
- Support for configuring all CLI options via environment variables (e.g. `SIA_FONT`, `SIA_BG_COLOR`).  
- Short flags for frequently used options (e.g. `-F`/`--font`, `-O`/`--output`).  
- `--input` now accepts a file path and reads preview text from the file.  
- Added dependency on the `rgb` crate for color parsing, replacing the custom `Color` type.  

### Changed
- Renamed project and binary from `fontpreview` to `sia`.  
- Removed global default constants (`VERSION`, `DEFAULT_SIZE`, etc.) and inlined defaults in clap argument definitions.  
- Replaced custom `ParseError` and error-handling with a unified `SiaError` enum; parsing errors now reported as `SiaError::Parse`.  
- Swapped out the hand-rolled `Color` struct for `rgb::RGBA8`; updated functions to return `SiaError`.  
- Clippy/style clean-ups: loop refactorings, import removals, use of array args, minor signature tweaks.  

### Removed
- Custom `Color` struct and `ParseError` type.  
- Global constants for defaults in source code.  

### Fixed
- Corrected documentation for `--input` to note it accepts either literal text or a file path.  
- Hex-color parsing now properly handles both 6-digit (`#RRGGBB`) and 8-digit (`#RRGGBBAA`) formats and reports invalid codes clearly.  
- Improved font-name detection logic for multi-word names and enhanced related error messages.  

## [0.1.0] – 2025-05-13

### Added
- Initial project skeleton  
  - `.gitignore` to ignore `.DS_Store`  
  - Luarocks config (`.luarocks/config-5.1.lua`, `default-lua-version.lua`)  
  - Vendored Lua modules (`lua-vips` for image processing, `luafilesystem` for FS access) under `lua_modules`
- New CLI tool **fontpeek**  
  - Shebang autodetection with `#!/usr/bin/env luajit`  
  - Bundled dependencies via `package.path` injection  
- Rich command-line interface  
  - `-i/--input`, `-o/--output`  
  - `--size`, `--font-size`, `--bg-color`, `--fg-color`  
  - `--bg-alpha`, `--fg-alpha`, `--preview-text`  
  - `-h/--help`, `--version`  
- Color and alpha support  
  - Hex parsing of `#RGB` and `#RRGGBB`  
  - Alpha blending for both background and text  
- Background generation in sRGB + alpha  
  - `vips.Image.black` + `draw_rect({r,g,b,a},…,{fill=true})`  
- Text rendering and layout  
  - `vips.Image.text` (multiline, `fontfile`, `dpi`, `wrap`, `align`)  
  - Automatic centering via computed `x`,`y` offsets and `composite2(...,{x,y})`  
- Font metadata and validation  
  - OS-specific font‐name resolution (PowerShell on Windows, `fc-scan` on Unix)  
  - Latin-script support check (`fc-scan --format %{lang}`), with warning if absent  

### Changed
- Renamed script from `fontpreview.lua` to **fontpeek**  
- Default style updates  
  - Background default `#FFFFFF` (was `#500000`)  
  - Font size default `23` pt (was `38`)  
- Command-line parsing improvements  
  - Support absolute and tilde-prefixed input paths  
  - Proper error on unknown options  
- Module-loading streamlined to include `lua_modules/share/lua/5.1`  
- Ensured background and text images share the same `format()` before compositing  

### Fixed
- Robust font‐family extraction: strip commas/spaces and pick first proper noun  
- Folded edge-case bugfixes (hex parsing, OS detection, alpha handling) into initial implementations  

---

[Unreleased]: https://github.com/philocalyst/sia/compare/v0.2.0…HEAD
[0.2.0]: https://github.com/philocalyst/sia/compare/v1.1.0…v0.2.0  
[0.1.0]: https://github.com/philocalyst/sia/compare/…v0.1.0 
