"""
A helper script to integrate the generated geo data into the app.
"""

import os
import shutil

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
SOURCE_DIR = os.path.join(SCRIPT_DIR, "out")
GEO_ASSETS_DEST_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, "../assets/geo"))
TRANSLATIONS_DEST_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, "../locales"))

GEO_ASSETS_TO_COPY = [
  "cities.rbush.json",
  "countries.rbush.json",
  "geometry.json",
  "geometry.rbush.json",
  "states-provinces-lines.json",
  "states-provinces-lines.rbush.json",
]

TRANSLATIONS_TO_COPY = [
  "countries.pot",
  "cities.pot",
]

if not os.path.exists(GEO_ASSETS_DEST_DIR):
  os.makedirs(GEO_ASSETS_DEST_DIR)

for f in GEO_ASSETS_TO_COPY:
  src = os.path.join(SOURCE_DIR, f)
  dst = os.path.join(GEO_ASSETS_DEST_DIR, f)
  prefix_len = len(os.path.commonprefix((src, dst)))

  print "Copying {} to {}".format(src[prefix_len:], dst[prefix_len:])

  shutil.copyfile(src, dst)

for f in TRANSLATIONS_TO_COPY:
  src = os.path.join(SOURCE_DIR, f)
  dst = os.path.join(TRANSLATIONS_DEST_DIR, f)
  prefix_len = len(os.path.commonprefix((src, dst)))

  print "Copying {} to {}".format(src[prefix_len:], dst[prefix_len:])

  shutil.copyfile(src, dst)
