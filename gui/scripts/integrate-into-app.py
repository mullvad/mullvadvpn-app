"""
A helper script to integrate the generated geo data into the app.
"""

import os
from os import path
from distutils import dir_util
import shutil

SCRIPT_DIR = path.dirname(path.realpath(__file__))
SOURCE_DIR = path.join(SCRIPT_DIR, "out")
GEO_ASSETS_DEST_DIR = path.realpath(path.join(SCRIPT_DIR, "../assets/geo"))
TRANSLATIONS_SOURCE_DIR = path.join(SOURCE_DIR, "locales")
TRANSLATIONS_DEST_DIR = path.realpath(path.join(SCRIPT_DIR, "../locales"))

GEO_ASSETS_TO_COPY = [
  "cities.rbush.json",
  "countries.rbush.json",
  "geometry.json",
  "geometry.rbush.json",
  "states-provinces-lines.json",
  "states-provinces-lines.rbush.json",
]

if not path.exists(GEO_ASSETS_DEST_DIR):
  os.makedirs(GEO_ASSETS_DEST_DIR)

for f in GEO_ASSETS_TO_COPY:
  src = path.join(SOURCE_DIR, f)
  dst = path.join(GEO_ASSETS_DEST_DIR, f)
  prefix_len = len(path.commonprefix((src, dst)))

  print "Copying {} to {}".format(src[prefix_len:], dst[prefix_len:])

  shutil.copyfile(src, dst)


print "Copying subtree {} -> {}".format(TRANSLATIONS_SOURCE_DIR, TRANSLATIONS_DEST_DIR)

dir_util.copy_tree(TRANSLATIONS_SOURCE_DIR, TRANSLATIONS_DEST_DIR)
