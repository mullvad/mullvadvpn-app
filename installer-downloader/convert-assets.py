#!/usr/bin/env python

# Convert svg assets to png assets

import os

os.add_dll_directory(r'C:\Program Files\UniConvertor-2.0rc5\dlls')
os.environ['path'] += r';C:\Program Files\UniConvertor-2.0rc5\dlls'

from cairosvg import svg2png

svg2png(url="src/logo-icon.svg", write_to="src/logo-icon.png", output_width=32)
svg2png(url="src/logo-text.svg", write_to="src/logo-text.png", output_width=122)
