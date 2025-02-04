#!/usr/bin/env python

# Convert svg assets to png assets

from cairosvg import svg2png

svg2png(url="src/logo-icon.svg", write_to="src/logo-icon.png", output_width=32)
svg2png(url="src/logo-text.svg", write_to="src/logo-text.png", output_width=122)
