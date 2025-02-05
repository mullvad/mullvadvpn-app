#!/usr/bin/env python

# Convert svg assets to png assets

from cairosvg import svg2png

svg2png(url="assets/logo-icon.svg", write_to="assets/logo-icon.png", output_width=32)
svg2png(url="assets/logo-text.svg", write_to="assets/logo-text.png", output_width=122)
