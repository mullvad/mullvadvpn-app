#!/usr/bin/env python3

"""
A helper script to parse NSLocalizedString in Swift source files and extract and merge new
translations with the existing ones.
"""

import os
from os import path
from subprocess import Popen, PIPE
import shutil
from nslocalized import StringTable

# Current script dir
SCRIPT_DIR = path.dirname(path.realpath(__file__))

# Path to directory with source files (Swift)
SOURCE_PATH = path.join(SCRIPT_DIR, "MullvadVPN")

# Path to directory with base localizations
BASE_LANGUAGE_PATH = path.join(SOURCE_PATH, "en.lproj")

# Path to directory with the output of genstrings tool.
GENSTRINGS_OUTPUT_PATH = path.join(SCRIPT_DIR, "genstrings-out")

# Output encoding for strings files.
#
# By default genstrings tool outputs text in utf-16, which git recognizes as binary, therefore
# rendering diff capability useless.
#
# Store localization files in utf-8 to fix this. Xcode automatically transcodes localization
# files to utf-16 during the build phase.
OUTPUT_ENCOODING = "utf_8"


def check_file_extension(file, expected_extension):
  """
  Returns True if the file extension matches the expected one.
  """
  (_basename, ext) = os.path.splitext(file)
  return ext == expected_extension


def get_source_files():
  """
  Find all Swift source files recursively.
  """
  results = []
  for root, _dirs, files in os.walk(SOURCE_PATH):
    for file in files:
      if check_file_extension(file, ".swift"):
        results.append(path.join(root, file))
  return results


def get_strings_files(dir_path):
  """
  Find all .strings files within the given directory.
  """
  results = []
  for file in os.listdir(dir_path):
    if check_file_extension(file, ".strings"):
      results.append(file)
  return results


def create_empty_output_dir():
  """
  Creates empty directory for output of genstrings tool.
  """
  # Wipe out old files
  delete_output_dir()

  # Re-create out directory
  os.mkdir(GENSTRINGS_OUTPUT_PATH)


def delete_output_dir():
  """
  Delete directory used for genstrings output
  """
  if path.exists(GENSTRINGS_OUTPUT_PATH):
    shutil.rmtree(GENSTRINGS_OUTPUT_PATH)


def extract_translations():
  """
  Extract translations from sources using genstrings tool.
  """

  # Get Swift source files
  source_files = get_source_files()

  # Genstrings utility comes with Xcode and used for extracting the localizable strings from source
  # files and producing string tables.
  args = (
    "genstrings", "-o", GENSTRINGS_OUTPUT_PATH,
    *source_files
  )
  (exit_code, errors) = run_program(*args)

  if exit_code == 0:
    print("Genstrings finished without errors.")
  else:
    print("Genstrings exited with {}: {}".format(exit_code, errors.decode().strip()))


def merge_translations():
  """
  Merge string tables, delete stale ones and copy new ones.
  """

  # Existing string tables
  existing_string_tables = get_strings_files(BASE_LANGUAGE_PATH)

  # Newly generated string tables
  new_string_tables = get_strings_files(GENSTRINGS_OUTPUT_PATH)

  # String tables that will be merged
  merge_string_tables = []

  # Detect new string tables
  for table_name in new_string_tables:
    if table_name not in existing_string_tables:
      src = path.join(GENSTRINGS_OUTPUT_PATH, table_name)
      dst = path.join(BASE_LANGUAGE_PATH, table_name)

      print("Copying {} to {}".format(src, dst))
      new_table = StringTable.read(src)
      new_table.write(dst, encoding=OUTPUT_ENCOODING)

  # Detect removed string tables
  for table_name in existing_string_tables:
    if table_name in new_string_tables:
      merge_string_tables.append(table_name)
    else:
      filepath = path.join(BASE_LANGUAGE_PATH, table_name)

      print("Removing {}".format(filepath))
      os.unlink(filepath)

  # Merge remaining string tables
  for table_name in merge_string_tables:
    new_table_path = path.join(GENSTRINGS_OUTPUT_PATH, table_name)
    base_table_path = path.join(BASE_LANGUAGE_PATH, table_name)

    merge_two_tables(base_table_path, new_table_path)


def merge_two_tables(base_table_path, new_table_path):
  """
  Merge new table into base table.
  """
  # Existing string table previously generated from sources
  base_table = StringTable.read(base_table_path)

  # New string table generated from sources
  new_table = StringTable.read(new_table_path)

  print("Merging {} into {}".format(new_table_path, base_table_path))

  # Iterate through newly generated table and preserve existing translations.
  for new_key in new_table.strings:
    base_entry = base_table.lookup(new_key)
    new_entry = new_table.lookup(new_key)
    if base_entry is not None:
      new_entry.target = base_entry.target

  print("Write {} on disk.".format(base_table_path))
  new_table.write(base_table_path, encoding=OUTPUT_ENCOODING)


def run_program(*args):
  """
  Run program and return a tuple with returncode and errors.
  """
  with Popen(args, stdin=PIPE, stdout=PIPE, stderr=PIPE) as subproc:
    print("Run: {}".format(' '.join(args)))

    errors = subproc.communicate()[1]
    return (subproc.returncode, errors)


def main():
  """
  Program entry
  """
  create_empty_output_dir()
  extract_translations()
  merge_translations()
  delete_output_dir()


main()
