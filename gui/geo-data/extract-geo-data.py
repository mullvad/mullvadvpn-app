"""
This module forms a geo json of highly populated cities in the world
"""

from os import path, makedirs
import json
from polib import POFile, POEntry
from subprocess import Popen, PIPE

# import order is important, see https://github.com/Toblerity/Shapely/issues/553
from shapely.geometry import shape, mapping
import fiona

SCRIPT_DIR = path.dirname(path.realpath(__file__))
OUT_DIR = path.join(SCRIPT_DIR, "out")

POPULATION_MAX_FILTER = 50000

def get_shape_path(dataset_name):
  return path.join(SCRIPT_DIR, dataset_name, dataset_name + ".shp")

def extract_cites():
  input_path = get_shape_path("ne_50m_populated_places_simple")
  output_path = path.join(OUT_DIR, "cities.json")

  props_to_keep = frozenset(["scalerank", "name", "latitude", "longitude"])

  features = []
  with fiona.collection(input_path, "r") as source:
    for feat in source:
      props = feat["properties"]
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

      # lowercase all keys
      props = dict((k.lower(), v) for k, v in feat["properties"].iteritems())

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
  output_path = path.join(OUT_DIR, "countries.pot")

  pot = POFile(encoding='UTF-8')

  with fiona.open(input_path) as source:
    for feat in source:
      # lowercase all keys
      props = dict((k.lower(), v) for k, v in feat["properties"].iteritems())

      entry = POEntry(
        msgid=props["name"],
        msgstr=u"",
        occurrences=[(input_basename, feat["id"])]
      )
      pot.append(entry)

  pot.save(output_path)
  print "Extracted {} countries to {}".format(len(pot), output_path)

def extract_cities_pot():
  input_path = get_shape_path("ne_50m_populated_places_simple")
  input_basename = path.basename(input_path)
  output_path = path.join(OUT_DIR, "cities.pot")

  pot = POFile(encoding='UTF-8')

  with fiona.open(input_path) as source:
    for feat in source:
      props = feat["properties"]
      if props["pop_max"] >= POPULATION_MAX_FILTER:
        entry = POEntry(
          msgid=props["name"],
          msgstr=u"",
          comment=u"{} {}".format(props["adm0name"], props["adm0_a3"]),
          occurrences=[(input_basename, feat["id"])]
        )
        pot.append(entry)

  pot.save(output_path)
  print "Extracted {} cities to {}".format(len(pot), output_path)


# ensure output path exists
if not path.exists(OUT_DIR):
  makedirs(OUT_DIR)

# extract all data
extract_cites()
extract_countries()
extract_geometry()
extract_provinces_and_states_lines()
extract_countries_pot()
extract_cities_pot()
