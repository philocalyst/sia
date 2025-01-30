# Welcome

fontpeek is a command line tool that lets you view a preview of the fonts in your home directory, with a seriously customizable set of options that allow YOU to view YOUR FONTS however YOU LIKE. 

Tons of credit to the inspiration and base that allowed me to build this script: https://github.com/sdushantha/fontpreview.

## Setup

- In order to run this tool, you'll need:
    - The Lua runtime
    - The libvips shared library
    - The luarocks package manager
    - The lua-vips library
    Automagically?
    `brew install vips lua luarocks`
    `luarocks install lua-vips`


Funny enough, with the default string you can't even see the benefits, it needs to be at least double that length for libvips to really start flexing. Realized this halfway through, but happy to offer a feature-complete font-previewer.
