"""
This module forms a topojson with geometries of all countries in the world
"""

import os
import json
from subprocess import Popen, PIPE

# import order is important, see https://github.com/Toblerity/Shapely/issues/553
from shapely.geometry import shape, mapping
import fiona

script_dir = os.path.dirname(os.path.realpath(__file__))
dataset_name = "ne_50m_admin_0_countries"
input_path = os.path.join(script_dir, dataset_name, dataset_name + ".shp")
output_path = os.path.join(script_dir, "geometry.json")

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
if p.returncode != 0:
  print "geo2topo exited with {}. {}".format(p.returncode, errors.decode('utf-8').strip())
