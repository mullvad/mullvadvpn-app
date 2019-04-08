"""
A helper script to integrate the generated geo data into the app.
"""

import os
from os import path
from subprocess import Popen, PIPE
import shutil
import colorful as c

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

TRANSLATIONS_TO_COPY = [
  "cities.po",
  "countries.po"
]

TRANSLATIONS_TO_MERGE = [
  "relay-locations.po"
]


def get_common_path(src, dst):
  prefix_len = len(path.commonprefix((src, dst)))
  return (src[prefix_len:], dst[prefix_len:])


if not path.exists(GEO_ASSETS_DEST_DIR):
  os.makedirs(GEO_ASSETS_DEST_DIR)

for f in GEO_ASSETS_TO_COPY:
  src = path.join(SOURCE_DIR, f)
  dst = path.join(GEO_ASSETS_DEST_DIR, f)

  print "Copying {} to {}".format(*get_common_path(src, dst))

  shutil.copyfile(src, dst)

for f in os.listdir(TRANSLATIONS_SOURCE_DIR):
  src = path.join(TRANSLATIONS_SOURCE_DIR, f)
  dst = path.join(TRANSLATIONS_DEST_DIR, f)

  if path.isdir(src):
    for f in os.listdir(src):
      src_po = path.join(src, f)
      dst_po = path.join(dst, f)

      if f in TRANSLATIONS_TO_COPY:
        print "Copying {} to {}".format(*get_common_path(src_po, dst_po))
        shutil.copyfile(src_po, dst_po)
      elif f in TRANSLATIONS_TO_MERGE:
        if path.exists(dst_po):
          pot_basename = path.basename(path.splitext(dst_po)[0])
          pot_path = path.join(TRANSLATIONS_DEST_DIR, pot_basename + ".pot")

          # shutil.copy(src_po, dst_po)
          #
          # p = Popen(
          #   ["msgmerge", "--update", "--no-fuzzy-matching", dst_po, pot_path],
          #   stdin=PIPE, stdout=PIPE, stderr=PIPE
          # )
          # errors = p.communicate()[1]
          # if p.returncode == 0:
          #   print c.green("Merged {} and {}".format(*get_common_path(src_po, dst_po)))
          # else:
          #   print c.red("msgmerge exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip()))
          #
          # p = Popen(["msgcat", src_po, dst_po, "--output-file", dst_po])
          # errors = p.communicate()[1]
          # if p.returncode == 0:
          #   print c.green("Concatenated {} -> {}".format(*get_common_path(src_po, dst_po)))
          # else:
          #   print c.red("msgcat exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip()))
        else:
          shutil.copy(src_po, dst_po)
      else:
        print c.orange("Unexpected file: {}".format(src_po))


  else:
    print "Copying {} to {}".format(*get_common_path(src, dst))
    shutil.copyfile(src, dst)
