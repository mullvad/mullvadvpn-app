#!/usr/bin/env python3
"""
This module adds relay location city and country names to relay-locations.pot
"""

import os
from os import path
import json
import urllib.request
from polib import POFile, POEntry
import colorful as c

SCRIPT_DIR = path.dirname(path.realpath(__file__))

# The output directory for the generated content
OUT_DIR = path.join(SCRIPT_DIR, "out")

# the directory with the generated localizations content
LOCALE_OUT_DIR = path.join(OUT_DIR, "locales")

# Relay locations gettext catalogue template filename (.pot)
RELAY_LOCATIONS_POT_FILENAME = "relay-locations.pot"


def extract_relay_translations():
  try:
    response = request_relays()
  except Exception as e:
    print(c.red("Failed to fetch the relays list: {}".format(e)))
    raise

  locations = response.get("locations")
  countries = structure_locations(locations)

  extract_relay_locations_pot(countries)


def structure_locations(locations):
  countries = {}

  for location_key in locations:
    location = locations.get(location_key)
    country_name = location.get("country")
    city_name = location.get("city")

    if not "-" in location_key:
      print("Location key incorrectly formatted: {}".format(location_key))
      continue

    country_code, city_code = location_key.split("-")

    if country_name is None:
      print("Country name missing for {}".format(location_key))
      continue

    if city_name is None:
      print("City name missing for {}".format(location_key))
      continue

    if country_code not in countries:
      countries[country_code] = {"name": country_name, "cities": {}}

    country = countries[country_code]
    cities = country["cities"]
    if city_code not in cities:
      cities[city_code] = city_name
    else:
      print("There are multiple entries for {} in {}".format(city_name, country_name))

  return countries


def extract_relay_locations_pot(countries):
  pot = POFile(encoding='utf-8', check_for_duplicates=True)
  pot.metadata = {"Content-Type": "text/plain; charset=utf-8"}
  output_path = path.join(LOCALE_OUT_DIR, RELAY_LOCATIONS_POT_FILENAME)

  print("Generating {}".format(output_path))

  for country_code in countries:
    country = countries[country_code]
    entry = POEntry(
      msgid=country["name"],
      msgstr="",
      comment=country_code.upper()
    )
    pot.append(entry)

    cities = country["cities"]
    for city_code in cities:
      entry = POEntry(
        msgid=cities[city_code],
        msgstr="",
        comment="{} {}".format(country_code.upper(), city_code.upper())
      )

      try:
        pot.append(entry)
      except ValueError as err:
        print(c.orange("Cannot add an entry: {}".format(err)))

  pot.save(output_path)


def request_relays():
  request = urllib.request.Request("https://api.mullvad.net/app/v1/relays")
  with urllib.request.urlopen(request) as connection:
    return json.load(connection)


# Program main()

def main():
  # ensure output path exists
  if not path.exists(OUT_DIR):
    os.makedirs(OUT_DIR)

  # ensure locales output path exists
  if not path.exists(LOCALE_OUT_DIR):
    os.makedirs(LOCALE_OUT_DIR)

  # extract translations
  extract_relay_translations()

main()
