const path = require('path');

module.exports = {
  getProjectRoots: () => {
    const roots = [];
    roots.push(__dirname);
    roots.push(path.resolve(__dirname, './app'));
    return roots;
  },
};
