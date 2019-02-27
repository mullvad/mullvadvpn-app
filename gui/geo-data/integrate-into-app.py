"""
A helper script to integrate the generated geo data into the app.
"""

import os
import shutil

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
SOURCE_DIR = os.path.join(SCRIPT_DIR, "out")
DESTINATION_DIR = os.path.realpath(os.path.join(SCRIPT_DIR, "../src/assets/geo"))

FILES_TO_COPY = [
  "cities.rbush.json",
  "countries.rbush.json",
  "geometry.json",
  "geometry.rbush.json",
  "states-provinces-lines.json",
  "states-provinces-lines.rbush.json"
]

if not os.path.exists(DESTINATION_DIR):
  os.makedirs(DESTINATION_DIR)

for f in FILES_TO_COPY:
  src = os.path.join(SOURCE_DIR, f)
  dst = os.path.join(DESTINATION_DIR, f)
  prefix_len = len(os.path.commonprefix((src, dst)))

  print "Copying {} to {}".format(src[prefix_len:], dst[prefix_len:])

  shutil.copyfile(src, dst)
