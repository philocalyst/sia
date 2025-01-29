# Welcome

First off, credit to the inspiration and base that allowed me to build this script: https://github.com/sdushantha/fontpreview. This is meant to be a modernization of that script that supports a very different kind of workflow and set of tools (macOS user here!). This means a broader set of input and output formats along with richer options. The tool does just one thing: convert fonts to images.

## Setup
- In order to run this tool, you'll need:
    - The Lua runtime
    - The libvips shared library
    - The luarocks package manager
    - The lua-vips library
    
    Automagically?
    `brew install vips lua luarocks`
    `luarocks install lua-vips`
