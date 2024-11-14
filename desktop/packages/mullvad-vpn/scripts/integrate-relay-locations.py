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

# The directory with the existing app localizations
APP_TRANSLATIONS_PATH = path.realpath(path.join(SCRIPT_DIR, "../locales"))

def merge_relay_locations_catalogue_template():
  existing_pot_file = path.join(APP_TRANSLATIONS_PATH, RELAY_LOCATIONS_POT_FILENAME)
  generated_pot_file = path.join(GENERATED_TRANSLATIONS_PATH, RELAY_LOCATIONS_POT_FILENAME)

  merge_gettext_catalogues(existing_pot_file, generated_pot_file)


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
  merge_relay_locations_catalogue_template()

main()
