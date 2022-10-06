//
// Script that generates r-trees for geo data.
// run with `npm exec ts-node geo-data/prepare-rtree.ts`
//

import * as fs from 'fs';
import * as path from 'path';
import { Topology, GeometryCollection } from 'topojson-specification';
import RBush from 'rbush';

interface GeometryTopologyObjects {
  [key: string]: any;
  geometry: GeometryCollection;
}

function main() {
  const GEOMETRY_DATA_FILES = ['geometry', 'states-provinces-lines'];
  const OUTPUT_DIR = path.join(__dirname, 'out');

  for (const name of GEOMETRY_DATA_FILES) {
    const source = path.join(OUTPUT_DIR, `${name}.json`);
    const destination = path.join(OUTPUT_DIR, `${name}.rbush.json`);

    try {
      processGeometry(source, destination);
    } catch (e) {
      const error = e as Error;
      console.error(`Failed to process ${name}: ${error.message}`);
    }
  }
}

function processGeometry(source: string, destination: string) {
  const collection = JSON.parse(
    fs.readFileSync(source, { encoding: 'utf8' }),
  ) as Topology<GeometryTopologyObjects>;

  const { geometry } = collection.objects;
  const treeData = geometry.geometries.map((object, i) => {
    if (!object.bbox) {
      throw new Error(`Expected a geometry at index ${i} to have a bbox property.`);
    }

    const [minX, minY, maxX, maxY] = object.bbox;
    return {
      ...object,
      minX,
      minY,
      maxX,
      maxY,
    };
  });

  const tree = new RBush();
  tree.load(treeData);
  fs.writeFileSync(destination, JSON.stringify(tree.toJSON()));

  console.log(`Saved a rbush to ${destination}`);
}

main();
