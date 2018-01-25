"""
This module forms a geo json of highly populated cities in the world
"""

import os
import json
import fiona

script_dir = os.path.dirname(os.path.realpath(__file__))
dataset_name = "ne_50m_populated_places_simple"
input_path = os.path.join(script_dir, dataset_name, dataset_name + ".shp")
output_path = os.path.join(script_dir, "cities.json")

props_to_keep = frozenset(["scalerank", "name", "latitude", "longitude"])

features = []
with fiona.collection(input_path, "r") as source:
  for feat in source:
    props = feat["properties"]
    if props["scalerank"] < 8:
      for k in frozenset(props) - props_to_keep:
        del props[k]
      features.append(feat)

my_layer = {
  "type": "FeatureCollection",
  "features": features
}

with open(output_path, "w") as f:
  f.write(json.dumps(my_layer))
