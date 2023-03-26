#!/usr/bin/env python3

"""
A helper script to integrate the generated geo data into the app.
"""

import os
from os import path
from subprocess import Popen, PIPE
import shutil
import colorful as c

SCRIPT_DIR = path.dirname(path.realpath(__file__))

# The name of the relay locations gettext catalogue template file
RELAY_LOCATIONS_POT_FILENAME = "relay-locations.pot"

# The directory with the generated content
GENERATED_CONTENT_OUTPUT_PATH = path.join(SCRIPT_DIR, "out")

# The directory with the generated localizations content
GENERATED_TRANSLATIONS_PATH = path.join(GENERATED_CONTENT_OUTPUT_PATH, "locales")

# The directory with the app's geo assets
APP_GEO_ASSETS_PATH = path.realpath(path.join(SCRIPT_DIR, "../assets/geo"))

# The directory with the existing app localizations
APP_TRANSLATIONS_PATH = path.realpath(path.join(SCRIPT_DIR, "../locales"))

# Geo assets for copying from generated content folder into the app folder
GEO_ASSETS_TO_COPY = [
  "geometry.json",
  "geometry.rbush.json",
  "states-provinces-lines.json",
  "states-provinces-lines.rbush.json",
]

# The filenames of gettext catalogues that should be merged using msgcat
TRANSLATIONS_TO_MERGE = [
  "relay-locations.po"
]


def copy_geo_assets():
  for f in GEO_ASSETS_TO_COPY:
    src = path.join(GENERATED_CONTENT_OUTPUT_PATH, f)
    dst = path.join(APP_GEO_ASSETS_PATH, f)

    print("Copying {} to {}".format(src, dst))

    shutil.copyfile(src, dst)


def merge_relay_locations_catalogue_template():
  existing_pot_file = path.join(APP_TRANSLATIONS_PATH, RELAY_LOCATIONS_POT_FILENAME)
  generated_pot_file = path.join(GENERATED_TRANSLATIONS_PATH, RELAY_LOCATIONS_POT_FILENAME)

  merge_gettext_catalogues(existing_pot_file, generated_pot_file)


def copy_and_merge_translations():
  for f in os.listdir(GENERATED_TRANSLATIONS_PATH):
    src = path.join(GENERATED_TRANSLATIONS_PATH, f)
    dst = path.join(APP_TRANSLATIONS_PATH, f)

    if path.isdir(src):
      merge_single_locale_folder(src, dst)


def merge_single_locale_folder(src, dst):
  for f in os.listdir(src):
    src_po = path.join(src, f)
    dst_po = path.join(dst, f)

    if f in TRANSLATIONS_TO_MERGE:
      # merge ../locales/*/file.po with ./out/locales/*/file.po
      # use existing translation to resolve conflicts
      merge_gettext_catalogues(dst_po, src_po)
    else:
      print(c.orange("Unexpected file: {}".format(src_po)))


def merge_gettext_catalogues(existing_catalogue_file, generated_catalogue_file):
  if path.exists(existing_catalogue_file):
    args = (
      existing_catalogue_file, generated_catalogue_file,

      "--output-file", existing_catalogue_file,

      # ensure that the first occurrence takes precedence in merge conflict
      "--use-first",

      # sort by msgid
      "--sort-output",

      # disable wrapping long strings because crowdin does not do that
      "--no-wrap"
    )

    (exit_code, errors) = run_program("msgcat", *args)

    if exit_code == 0:
      print(c.green("Merged {} into {}.".format(generated_catalogue_file, existing_catalogue_file)))
    else:
      print(c.red("msgcat exited with {}: {}".format(exit_code, errors.decode().strip())))
  else:
    print(c.orange("The existing catalogue does not exist. Copying {} to {}")
      .format(generated_catalogue_file, existing_catalogue_file))
    shutil.copyfile(generated_catalogue_file, existing_catalogue_file)


def run_program(*args):
  with Popen(args, stdin=PIPE, stdout=PIPE, stderr=PIPE) as subproc:
    print("Run: {}".format(' '.join(args)))

    errors = subproc.communicate()[1]
    return (subproc.returncode, errors)


# Program main()

def main():
  if not path.exists(APP_GEO_ASSETS_PATH):
    os.makedirs(APP_GEO_ASSETS_PATH)

  copy_geo_assets()
  merge_relay_locations_catalogue_template()
  copy_and_merge_translations()

main()
