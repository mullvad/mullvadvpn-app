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

RELAY_LOCATIONS_POT_FILENAME = "relay-locations.pot"

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

def run_program(*args):
  p = Popen(args, stdin=PIPE, stdout=PIPE, stderr=PIPE)

  print u"Run: {}".format(' '.join(args))

  errors = p.communicate()[1]
  return (p.returncode, errors)

def copy_geo_assets():
  for f in GEO_ASSETS_TO_COPY:
    src = path.join(SOURCE_DIR, f)
    dst = path.join(GEO_ASSETS_DEST_DIR, f)

    print u"Copying {} to {}".format(*remove_common_prefix(src, dst))

    shutil.copyfile(src, dst)

def copy_and_merge_translations():
  for f in os.listdir(TRANSLATIONS_SOURCE_DIR):
    src = path.join(TRANSLATIONS_SOURCE_DIR, f)
    dst = path.join(TRANSLATIONS_DEST_DIR, f)

    if path.isdir(src):
      merge_single_locale_folder(src, dst)
    else:
      print u"Copying {} to {}".format(*remove_common_prefix(src, dst))
      shutil.copyfile(src, dst)

def merge_single_locale_folder(src, dst):
  for f in os.listdir(src):
    src_po = path.join(src, f)
    dst_po = path.join(dst, f)

    if f in TRANSLATIONS_TO_COPY:
      print u"Copying {} to {}".format(*remove_common_prefix(src_po, dst_po))
      shutil.copyfile(src_po, dst_po)
    elif f in TRANSLATIONS_TO_MERGE:
      if path.exists(dst_po):
        # merge ../locales/*/file.po with ./out/locales/*/file.po
        # existing translations applied on top of the generated ones
        (exit_code, errors) = run_msgcat(dst_po, src_po, dst_po)

        if exit_code == 0:
            print c.green(u"Merged {} into {}.".format(*remove_common_prefix(src_po, dst_po)))
        else:
          print c.red(u"msgcat exited with {}: {}".format(
            exit_code, errors.decode('utf-8').strip()))
      else:
        print c.orange(u"Nothing to merge. Copying {} to {}"
          .format(*remove_common_prefix(src_po, dst_po)))
        shutil.copy(src_po, dst_po)
    else:
      print c.orange(u"Unexpected file: {}".format(src_po))

def merge_relay_locations_pot():
  existing_pot_file = path.join(TRANSLATIONS_DEST_DIR, RELAY_LOCATIONS_POT_FILENAME)
  generated_pot_file = path.join(TRANSLATIONS_SOURCE_DIR, RELAY_LOCATIONS_POT_FILENAME)

  if path.exists(existing_pot_file):
    print u"Found the existing {}. Merging.".format(RELAY_LOCATIONS_POT_FILENAME)

    # merge the existing and generated relay-locations.pot
    (exit_code, errors) = run_msgcat(existing_pot_file, generated_pot_file, existing_pot_file)

    if exit_code == 0:
      print c.green(u"Merged {} into {}."
        .format(*remove_common_prefix(generated_pot_file, existing_pot_file)))
    else:
      print c.red(u"msgcat exited with {}: {}".format(exit_code, errors.decode('utf-8').strip()))
  else:
    print c.orange(u"Nothing to merge. Copying {} to {}"
      .format(*remove_common_prefix(generated_pot_file, existing_pot_file)))
    shutil.copy(generated_pot_file, existing_pot_file)


def run_msgcat(first_file, second_file, output_file):
  args = (
    first_file, second_file,
    "--output-file", output_file,

    # ensure that the first occurence takes precedence in merge conflict
    "--use-first",

    # sort by msgid
    "--sort-output",

    # disable wrapping long strings because crowdin does not do that
    "--no-wrap"
  )
  return run_program("msgcat", *args)

if not path.exists(GEO_ASSETS_DEST_DIR):
  os.makedirs(GEO_ASSETS_DEST_DIR)

copy_geo_assets()
merge_relay_locations_pot()
copy_and_merge_translations()
