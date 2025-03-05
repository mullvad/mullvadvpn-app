#!/usr/bin/env python

# Convert svg assets to png assets
# This must be done manually

from cairosvg import svg2png

svg2png(url="assets/logo-icon.svg", write_to="assets/logo-icon.png", output_width=32)
svg2png(url="assets/logo-text.svg", write_to="assets/logo-text.png", output_width=122)
svg2png(url="assets/alert-circle.svg", write_to="assets/alert-circle.png", output_width=32)
