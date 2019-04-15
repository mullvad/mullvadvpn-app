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

def remove_common_prefix(source, destination):
  prefix_len = len(path.commonprefix((source, destination)))
  return (source[prefix_len:], destination[prefix_len:])

def run_program(args):
  p = Popen(args, stdin=PIPE, stdout=PIPE, stderr=PIPE)

  print "Run: {}".format(' '.join(args))

  errors = p.communicate()[1]
  return (p.returncode, errors)

def copy_geo_assets():
  for f in GEO_ASSETS_TO_COPY:
    src = path.join(SOURCE_DIR, f)
    dst = path.join(GEO_ASSETS_DEST_DIR, f)

    print "Copying {} to {}".format(*remove_common_prefix(src, dst))

    shutil.copyfile(src, dst)

def copy_and_merge_translations():
  for f in os.listdir(TRANSLATIONS_SOURCE_DIR):
    src = path.join(TRANSLATIONS_SOURCE_DIR, f)
    dst = path.join(TRANSLATIONS_DEST_DIR, f)

    if path.isdir(src):
      merge_single_locale_folder(src, dst)
    else:
      print "Copying {} to {}".format(*remove_common_prefix(src, dst))
      shutil.copyfile(src, dst)

def merge_single_locale_folder(src, dst):
  for f in os.listdir(src):
    src_po = path.join(src, f)
    dst_po = path.join(dst, f)

    if f in TRANSLATIONS_TO_COPY:
      print "Copying {} to {}".format(*remove_common_prefix(src_po, dst_po))
      shutil.copyfile(src_po, dst_po)
    elif f in TRANSLATIONS_TO_MERGE:
      if path.exists(dst_po):
        pot_basename = path.basename(path.splitext(dst_po)[0])
        pot_path = path.join(TRANSLATIONS_DEST_DIR, pot_basename + ".pot")

        (msgmerge_code, msgmerge_errors) = run_program([
          "msgmerge", "--update", "--no-fuzzy-matching", dst_po, pot_path])

        if msgmerge_code == 0:
          (msgcat_code, msgcat_errors) = run_program([
            "msgcat", src_po, dst_po, "--output-file", dst_po])

          if msgcat_code == 0:
            print c.green("Merged and concatenated the catalogues.")
          else:
            print c.red("msgcat exited with {}: {}".format(
              msgcat_code, msgcat_errors.decode('utf-8').strip()))
        else:
          print c.red("msgmerge exited with {}: {}".format(
            msgmerge_code, msgmerge_errors.decode('utf-8').strip()))
      else:
        shutil.copy(src_po, dst_po)
    else:
      print c.orange("Unexpected file: {}".format(src_po))


if not path.exists(GEO_ASSETS_DEST_DIR):
  os.makedirs(GEO_ASSETS_DEST_DIR)

copy_geo_assets()
copy_and_merge_translations()
