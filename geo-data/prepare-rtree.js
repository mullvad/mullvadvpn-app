//
// Script that generates r-trees for geo data.
// run with `yarn babel-node geo-data/prepare-rtree.js`
//

import fs from 'fs';
import path from 'path';
import rbush from 'rbush';

const input = ['countries', 'cities', 'geometry', 'states-provinces-lines'];

for(const name of input) {
  const source = path.join(__dirname, `${name}.json`);
  const destination = path.join(__dirname, `${name}.rbush.json`);
  const data = fs.readFileSync(source);
  const collection = JSON.parse(data);

  let treeData;
  if(collection.type.toLowerCase() === 'topology') {
    const { geometry } = collection.objects;
    treeData = geometry.geometries.map(object => {
      const [minX, minY, maxX, maxY] = object.bbox;
      return {
        ...object,
        minX, minY, maxX, maxY
      };
    });
  } else if(collection.type.toLowerCase() === 'featurecollection') {
    treeData = collection.features.map((feat) => {
      const { coordinates } = feat.geometry;
      return { ...feat,
        minX: coordinates[0],
        minY: coordinates[1],
        maxX: coordinates[0],
        maxY: coordinates[1],
      };
    });
  }

  const tree = rbush();
  tree.load(treeData);
  fs.writeFileSync(destination, JSON.stringify(tree.toJSON()));

  console.log(`Saved a rbush tree at ${destination}`);
}

