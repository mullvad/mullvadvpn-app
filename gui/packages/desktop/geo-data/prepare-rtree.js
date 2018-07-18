//
// Script that generates r-trees for geo data.
// run with `yarn babel-node geo-data/prepare-rtree.js`
//

import fs from 'fs';
import path from 'path';
import rbush from 'rbush';

const geometryData = ['geometry', 'states-provinces-lines'];
const pointData = ['countries', 'cities'];

const output_dir = path.join(__dirname, 'out');

for (const name of geometryData) {
  const source = path.join(output_dir, `${name}.json`);
  const destination = path.join(output_dir, `${name}.rbush.json`);
  const collection = JSON.parse(fs.readFileSync(source));

  const { geometry } = collection.objects;
  const treeData = geometry.geometries.map((object) => {
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

for (const name of pointData) {
  const source = path.join(output_dir, `${name}.json`);
  const destination = path.join(output_dir, `${name}.rbush.json`);
  const collection = JSON.parse(fs.readFileSync(source));

  const treeData = collection.features.map((feat) => {
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
