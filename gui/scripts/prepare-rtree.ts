//
// Script that generates r-trees for geo data.
// run with `npx babel-node geo-data/prepare-rtree.js`
//

import * as fs from 'fs';
import * as path from 'path';
import { Topology, GeometryCollection } from 'topojson-specification';
import { GeoJSON } from 'geojson';
import rbush from 'rbush';

interface GeometryTopologyObjects {
  [key: string]: any;
  geometry: GeometryCollection;
}

function main() {
  const GEOMETRY_DATA_FILES = ['geometry', 'states-provinces-lines'];
  const POINT_DATA_FILES = ['countries', 'cities'];
  const OUTPUT_DIR = path.join(__dirname, 'out');

  for (const name of GEOMETRY_DATA_FILES) {
    const source = path.join(OUTPUT_DIR, `${name}.json`);
    const destination = path.join(OUTPUT_DIR, `${name}.rbush.json`);

    try {
      processGeometry(source, destination);
    } catch (error) {
      console.error(`Failed to process ${name}: ${error.message}`);
    }
  }

  for (const name of POINT_DATA_FILES) {
    const source = path.join(OUTPUT_DIR, `${name}.json`);
    const destination = path.join(OUTPUT_DIR, `${name}.rbush.json`);

    try {
      processPoints(source, destination);
    } catch (error) {
      console.error(`Failed to process ${name}: ${error.message}`);
    }
  }
}

function processGeometry(source: string, destination: string) {
  const collection = JSON.parse(fs.readFileSync(source, { encoding: 'utf8' })) as Topology<
    GeometryTopologyObjects
  >;

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

  const tree = rbush();
  tree.load(treeData);
  fs.writeFileSync(destination, JSON.stringify(tree.toJSON()));

  console.log(`Saved a rbush to ${destination}`);
}

function processPoints(source: string, destination: string) {
  const collection = JSON.parse(fs.readFileSync(source, { encoding: 'utf8' })) as GeoJSON;

  if (collection.type !== 'FeatureCollection') {
    throw new Error(
      `Invalid collection type ${collection.type} in ${source}. Expected FeatureCollection`,
    );
  }

  const treeData = collection.features.map((feat) => {
    if (feat.geometry.type !== 'Point') {
      throw new Error(`Invalid geometry in ${source}. Expected "Point", got ${feat.geometry.type}`);
    }

    const { coordinates } = feat.geometry;
    return {
      ...feat,
      minX: coordinates[0],
      minY: coordinates[1],
      maxX: coordinates[0],
      maxY: coordinates[1],
    };
  });

  const tree = rbush();
  tree.load(treeData);
  fs.writeFileSync(destination, JSON.stringify(tree.toJSON()));

  console.log(`Saved a rbush to ${destination}`);
}

main();
