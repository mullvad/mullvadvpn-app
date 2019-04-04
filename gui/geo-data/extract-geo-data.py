"""
This module forms a geo json of highly populated cities in the world
"""

import os
from os import path
import json, pprint
from polib import POFile, POEntry
from subprocess import Popen, PIPE

# import order is important, see https://github.com/Toblerity/Shapely/issues/553
from shapely.geometry import shape, mapping
import fiona

SCRIPT_DIR = path.dirname(path.realpath(__file__))
LOCALE_DIR = path.normpath(path.join(SCRIPT_DIR, "../locales"))
OUT_DIR = path.join(SCRIPT_DIR, "out")
LOCALE_OUT_DIR = path.join(OUT_DIR, "locales")

POPULATION_MAX_FILTER = 50000

def get_shape_path(dataset_name):
  return path.join(SCRIPT_DIR, dataset_name, dataset_name + ".shp")

def lower_dict_keys(input_dict):
  return dict((k.lower(), v) for k, v in input_dict.iteritems())

def convert_locale_ident(locale_ident):
  return locale_ident.replace("-", "_")

def get_locale_language(locale_ident):
  return locale_ident.split("-")[0]

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

  print "Extracted data to {}".format(output_path)


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

  print "Extracted data to {}".format(output_path)


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
    print "Extracted data to {}".format(output_path)
  else:
    print "geo2topo exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip())


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
    print "Extracted data to {}".format(output_path)
  else:
    print "geo2topo exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip())

def extract_countries_pot():
  input_path = get_shape_path("ne_50m_admin_0_countries")
  input_basename = path.basename(input_path)

  for locale in os.listdir(LOCALE_DIR):
    locale_dir = path.join(LOCALE_DIR, locale)
    locale_out_dir = path.join(LOCALE_OUT_DIR, locale)

    if os.path.isdir(locale_dir):
      with fiona.open(input_path) as source:
        pot = POFile(encoding='UTF-8')
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
            print u"Missing translation for {}".format(translated_name)
          else:
            raise ValueError(
              "Cannot find the translation for {}. Probe keys: {}"
                .format(locale, (name_key, name_alt_key) ))

          entry = POEntry(
            msgid=props["name"],
            msgstr=translated_name,
            occurrences=[(input_basename, feat["id"])]
          )
          pot.append(entry)

        pot.save(output_path)
        print "Extracted {} countries for {} to {}".format(len(pot), locale, output_path)

def extract_cities_pot():
  input_path = get_shape_path("ne_50m_populated_places")
  input_basename = path.basename(input_path)
  output_path = path.join(OUT_DIR, "cities.pot")

  for locale in os.listdir(LOCALE_DIR):
    locale_dir = path.join(LOCALE_DIR, locale)
    locale_out_dir = path.join(LOCALE_OUT_DIR, locale)

    if os.path.isdir(locale_dir):
      pot = POFile(encoding='UTF-8')
      output_path = path.join(locale_out_dir, "cities.po")

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
            elif props.get(name_alt_key) is not None:
              translated_name = props.get(name_alt_key)
            elif props.get(name_fallback) is not None:
              translated_name = props.get(name_fallback)
              print u"Missing translation for {}".format(translated_name)
            else:
              raise ValueError(
                "Cannot find the translation for {}. Probe keys: {}"
                  .format(locale, (name_key, name_alt_key) ))

            entry = POEntry(
              msgid=props["name"],
              msgstr=translated_name,
              occurrences=[(input_basename, feat["id"])]
            )
            pot.append(entry)

      pot.save(output_path)
      print "Extracted {} cities to {}".format(len(pot), output_path)


# ensure output path exists
if not path.exists(OUT_DIR):
  os.makedirs(OUT_DIR)

# extract all data
extract_cities()
extract_countries()
extract_geometry()
extract_provinces_and_states_lines()
extract_countries_pot()
extract_cities_pot()
