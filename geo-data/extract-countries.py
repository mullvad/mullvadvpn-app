"""
This module forms a geo json of all countries in the world
"""

import os
import json

# import order is important, see https://github.com/Toblerity/Shapely/issues/553
from shapely.geometry import shape, mapping
import fiona

script_dir = os.path.dirname(os.path.realpath(__file__))
dataset_name = "ne_50m_admin_0_countries"
input_path = os.path.join(script_dir, dataset_name, dataset_name + ".shp")
output_path = os.path.join(script_dir, "countries.json")

props_to_keep = frozenset(["name"])

features = []
with fiona.open(input_path) as source:
  for feat in source:
    geometry = feat["geometry"]

    # convert country polygon to centroid
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
