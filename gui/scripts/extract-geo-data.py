"""
This module forms a geo json of highly populated cities in the world
"""

import os
from os import path
import json
import urllib2
from subprocess import Popen, PIPE
from polib import POFile, POEntry
import colorful as c
from terminaltables import AsciiTable

# import order is important, see https://github.com/Toblerity/Shapely/issues/553
from shapely.geometry import shape, mapping
import fiona

SCRIPT_DIR = path.dirname(path.realpath(__file__))
LOCALE_DIR = path.normpath(path.join(SCRIPT_DIR, "../locales"))
OUT_DIR = path.join(SCRIPT_DIR, "out")
LOCALE_OUT_DIR = path.join(OUT_DIR, "locales")

POPULATION_MAX_FILTER = 50000

def extract_cities():
  input_path = get_shape_path("ne_50m_populated_places")
  output_path = path.join(OUT_DIR, "cities.json")

  props_to_keep = frozenset(["scalerank", "name", "latitude", "longitude"])

  features = []
  with fiona.collection(input_path, "r") as source:
    for feat in source:
      props = lower_dict_keys(feat["properties"])

      if props["pop_max"] >= POPULATION_MAX_FILTER:
        for k in frozenset(props) - props_to_keep:
          del props[k]
        features.append(feat)

  my_layer = {
    "type": "FeatureCollection",
    "features": features
  }

  with open(output_path, "w") as f:
    f.write(json.dumps(my_layer))

  print c.green("Extracted data to {}".format(output_path))


def extract_countries():
  input_path = get_shape_path("ne_50m_admin_0_countries")
  output_path = path.join(OUT_DIR, "countries.json")

  props_to_keep = frozenset(["name"])

  features = []
  with fiona.open(input_path) as source:
    for feat in source:
      geometry = feat["geometry"]

      # convert country polygon to point
      geometry.update(mapping(shape(geometry).representative_point()))

      props = lower_dict_keys(feat["properties"])
      for k in frozenset(props) - props_to_keep:
        del props[k]

      feat["properties"] = props

      features.append(feat)

  my_layer = {
    "type": "FeatureCollection",
    "features": features
  }

  with open(output_path, "w") as f:
    f.write(json.dumps(my_layer))

  print c.green("Extracted data to {}".format(output_path))


def extract_geometry():
  input_path = get_shape_path("ne_50m_admin_0_countries")
  output_path = path.join(OUT_DIR, "geometry.json")

  features = []
  with fiona.open(input_path) as source:
    for feat in source:
      del feat["properties"]
      geometry = feat["geometry"]
      feat["bbox"] = shape(geometry).bounds
      features.append(feat)

  my_layer = {
    "type": "FeatureCollection",
    "features": features
  }

  p = Popen(
    ['geo2topo', '-q', '1e5', 'geometry=-', '-o', output_path],
    stdin=PIPE, stdout=PIPE, stderr=PIPE
  )
  errors = p.communicate(input=json.dumps(my_layer))[1]
  if p.returncode == 0:
    print c.green("Extracted data to {}".format(output_path))
  else:
    print c.red("geo2topo exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip()))


def extract_provinces_and_states_lines():
  input_path = get_shape_path("ne_50m_admin_1_states_provinces_lines")
  output_path = path.join(OUT_DIR, "states-provinces-lines.json")

  features = []
  with fiona.open(input_path) as source:
    for feat in source:
      del feat["properties"]
      geometry = feat["geometry"]
      feat["bbox"] = shape(geometry).bounds
      features.append(feat)

  my_layer = {
    "type": "FeatureCollection",
    "features": features
  }

  p = Popen(
    ['geo2topo', '-q', '1e5', 'geometry=-', '-o', output_path],
    stdin=PIPE, stdout=PIPE, stderr=PIPE
  )
  errors = p.communicate(input=json.dumps(my_layer))[1]
  if p.returncode == 0:
    print c.green("Extracted data to {}".format(output_path))
  else:
    print c.red("geo2topo exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip()))


def extract_countries_po():
  input_path = get_shape_path("ne_50m_admin_0_countries")
  input_basename = path.basename(input_path)

  for locale in os.listdir(LOCALE_DIR):
    locale_dir = path.join(LOCALE_DIR, locale)
    locale_out_dir = path.join(LOCALE_OUT_DIR, locale)

    if os.path.isdir(locale_dir):
      with fiona.open(input_path) as source:
        po = POFile(encoding='UTF-8')
        po.metadata = { "Content-Type": "text/plain; charset=utf-8" }
        output_path = path.join(locale_out_dir, "countries.po")

        if not path.exists(locale_out_dir):
          os.makedirs(locale_out_dir)

        print "Generating {}/countries.po".format(locale)

        for feat in source:
          props = lower_dict_keys(feat["properties"])
          name_key = "_".join(("name", get_locale_language(locale)))
          name_alt_key = "_".join(("name", convert_locale_ident(locale)))
          name_fallback = "name"

          if props.get(name_key) is not None:
            translated_name = props.get(name_key)
          elif props.get(name_alt_key) is not None:
            translated_name = props.get(name_alt_key)
          elif props.get(name_fallback) is not None:
            translated_name = props.get(name_fallback)
            print c.orange(u" Missing translation for {}".format(translated_name))
          else:
            raise ValueError(
              "Cannot find the translation for {}. Probe keys: {}"
              .format(locale, (name_key, name_alt_key))
              )

          entry = POEntry(
            msgid=props["name"],
            msgstr=translated_name,
            occurrences=[(input_basename, feat["id"])]
          )
          po.append(entry)

        po.save(output_path)
        print c.green("Extracted {} countries for {} to {}".format(len(po), locale, output_path))


def extract_cities_po():
  input_path = get_shape_path("ne_50m_populated_places")
  input_basename = path.basename(input_path)

  stats = []

  for locale in os.listdir(LOCALE_DIR):
    locale_dir = path.join(LOCALE_DIR, locale)
    locale_out_dir = path.join(LOCALE_OUT_DIR, locale)

    if os.path.isdir(locale_dir):
      po = POFile(encoding='UTF-8')
      po.metadata = { "Content-Type": "text/plain; charset=utf-8" }
      output_path = path.join(locale_out_dir, "cities.po")
      hits = 0
      misses = 0

      if not path.exists(locale_out_dir):
        os.makedirs(locale_out_dir)

      print "Generating {}/cities.po".format(locale)

      with fiona.open(input_path) as source:
        for feat in source:
          props = lower_dict_keys(feat["properties"])

          if props["pop_max"] >= POPULATION_MAX_FILTER:
            name_key = "_".join(("name", get_locale_language(locale)))
            name_alt_key = "_".join(("name", convert_locale_ident(locale)))
            name_fallback = "name"

            if props.get(name_key) is not None:
              translated_name = props.get(name_key)
              hits += 1
            elif props.get(name_alt_key) is not None:
              translated_name = props.get(name_alt_key)
              hits += 1
            elif props.get(name_fallback) is not None:
              translated_name = props.get(name_fallback)
              print c.orange(u"  Missing translation for {}".format(translated_name))
              misses += 1
            else:
              raise ValueError(
                "Cannot find the translation for {}. Probe keys: {}"
                .format(locale, (name_key, name_alt_key))
                )

            entry = POEntry(
              msgid=props["name"],
              msgstr=translated_name,
              occurrences=[(input_basename, feat["id"])]
            )
            po.append(entry)

      po.save(output_path)
      print c.green("Extracted {} cities to {}".format(len(po), output_path))

      stats.append((locale, hits, misses))

  print_stats_table("Cities translations", stats)


def extract_relay_translations():
  try:
    response = request_relays()
  except Exception as e:
    print c.red("Failed to fetch the relays list: {}".format(e))
    raise

  result = response.get("result")
  if result is not None:
    countries = result.get("countries")
    if countries is None:
      raise Exception("Missing the countries field.")
  else:
    raise Exception("Missing the result field.")

  extract_relay_locations_pot(countries)
  translate_relay_locations_pot(countries)


def extract_relay_locations_pot(countries):
  pot = POFile(encoding='UTF-8')
  pot.metadata = { "Content-Type": "text/plain; charset=utf-8" }
  output_path = path.join(LOCALE_OUT_DIR, "relay-locations.pot")

  print "Generating relay-locations.pot"

  for country in countries:
    cities = country.get("cities")
    if cities is not None:
      for city in cities:
        city_name = city.get("name")
        if city_name is not None:
          entry = POEntry(
            msgid=city_name,
            msgstr=u"",
            comment=u"{} {}".format(country.get("code").upper(), city.get("code").upper())
          )
          pot.append(entry)
          print u"  {} ({})".format(city["name"], city["code"]).encode('utf-8')

  pot.save(output_path)


def prepare_stats_table_column(item):
  (locale, hits, misses) = item
  total = hits + misses
  hits_ratio = round(float(hits) / total * 100, 2) if total > 0 else 0

  misses_column = c.orange(str(misses)) if misses > 0 else c.green(str(misses))
  hits_column = c.green(str(hits))
  ratio_column = c.green(str(hits_ratio) + "%") if hits_ratio >= 80 else c.orange(str(hits_ratio))
  total_column = str(total)

  return (locale, hits_column, misses_column, ratio_column, total_column)

def print_stats_table(title, data):
  header = ("Locale", "Hits", "Misses", "% translated", "Total")
  color_data = map(prepare_stats_table_column, data)

  table = AsciiTable([header] + color_data)
  table.title = title

  for i in range(1, 5):
    table.justify_columns[i] = 'center'

  print ""
  print table.table
  print ""


def translate_relay_locations_pot(countries):
  place_translator = PlaceTranslator()
  stats = []

  for locale in os.listdir(LOCALE_DIR):
    locale_dir = path.join(LOCALE_DIR, locale)
    if path.isdir(locale_dir):
      print "Generating {}/relay-locations.po".format(locale)
      (hits, misses) = translate_relay_locations(place_translator, countries, locale)
      stats.append((locale, hits, misses))

  print_stats_table("Relay location translations", stats)


def translate_relay_locations(place_translator, countries, locale):
  po = POFile(encoding='UTF-8')
  po.metadata = { "Content-Type": "text/plain; charset=utf-8" }
  locale_out_dir = path.join(LOCALE_OUT_DIR, locale)
  output_path = path.join(locale_out_dir, "relay-locations.po")

  hits = 0
  misses = 0

  if not path.exists(locale_out_dir):
    os.makedirs(locale_out_dir)

  for country in countries:
    country_name = country.get("name")
    country_code = country.get("code")
    cities = country.get("cities")

    if cities is None:
      print c.orange(u"Skip {} ({}) because no cities were found.".format(country_name, country_code))
      continue

    for (index, city) in enumerate(cities):
      city_name = city.get("name")
      city_code = city.get("code")
      if city_name is None:
        raise ValueError("Missing the name field in city record.")

      # Make sure to append the US state back to the translated name of the city
      if country_code == "us":
        split = city_name.rsplit(",", 2)
        translated_name = place_translator.translate(locale, split[0].strip())

        if translated_name is not None and len(split) > 1:
          translated_name = u"{}, {}".format(translated_name, split[1].strip())
      else:
        translated_name = place_translator.translate(locale, city_name)

      # Default to empty string if no translation was found
      found_translation = translated_name is not None
      if found_translation:
        hits += 1
      else:
        translated_name = ""
        misses += 1

      log_message = u"  {} ({}) -> \"{}\"".format(city_name, city_code, translated_name).encode('utf-8')
      if found_translation:
        print c.green(log_message)
      else:
        print c.orange(log_message)

      entry = POEntry(
        msgid=city_name,
        msgstr=translated_name,
        comment=u"{} {}".format(country.get("code").upper(), city.get("code").upper())
      )
      po.append(entry)

  po.save(output_path)

  return (hits, misses)


### HELPERS ###

class PlaceTranslator(object):
  """
  This class provides facilities for translating places from one language to the other.
  It supports both English and
  """

  def __init__(self):
    super(PlaceTranslator, self).__init__()
    shape_path = get_shape_path("ne_10m_populated_places")
    self.source = fiona.open(shape_path, "r")

  def __del__(self):
    self.source.close()

  def translate(self, locale, english_city_name):
    """
    Lookup the populated places dataset for the city matching by name, par name or
    name representation in ASCII.

    When there is a match, the function looks for the translation using the given locale or using
    the language component of it.

    Returns None when either there is no match or there is no translation for the matched city.
    """
    preferred_locales = (get_locale_language(locale), convert_locale_ident(locale))
    match_prop_keys = ("name_" + x for x in preferred_locales)

    for feat in self.source:
      props = lower_dict_keys(feat["properties"])

      # namepar works for "Wien"
      # use nameascii to match "Sao Paolo"
      if props.get("name") == english_city_name or \
         props.get("namepar") == english_city_name or \
         props.get("nameascii") == english_city_name:
        for key in match_prop_keys:
          value = props.get(key)

          if value is not None:
            return value

        print c.orange(
          u"Missing translation for {} ({}). Probe keys: {}".format(english_city_name, locale, match_prop_keys)
          .encode('utf-8'))

    return None


def get_shape_path(dataset_name):
  return path.join(SCRIPT_DIR, dataset_name, dataset_name + ".shp")


def lower_dict_keys(input_dict):
  return dict((k.lower(), v) for k, v in input_dict.iteritems())


def convert_locale_ident(locale_ident):
  """
  Return the locale identifie converting dashes to underscores.

  Example: en-US becomes en_US
  """
  return locale_ident.replace("-", "_")


def get_locale_language(locale_ident):
  """
  Return a langauge code from locale identifier.

  Example #1: en-US, the function returns en
  Example #2: en, the function returns en
  """
  return locale_ident.split("-")[0]


def request_relays():
  data = json.dumps({"jsonrpc": "2.0", "id": "0", "method": "relay_list_v2"})
  headers = {"Content-Type": "application/json"}
  request = urllib2.Request("https://api.mullvad.net/rpc/", data, headers)
  return json.load(urllib2.urlopen(request))


# Program main()

def main():
  # ensure output path exists
  if not path.exists(OUT_DIR):
    os.makedirs(OUT_DIR)

  # ensure locales output path exists
  if not path.exists(LOCALE_OUT_DIR):
    os.makedirs(LOCALE_OUT_DIR)

  # extract geo data
  extract_cities()
  extract_countries()
  extract_geometry()
  extract_provinces_and_states_lines()

  # extract translations
  extract_countries_po()
  extract_cities_po()
  # extract_relay_translations()

main()
