const path = require('path');
const { task, series } = require('gulp');
const rimraf = require('rimraf');

const scripts = require('./tasks/scripts');
const assets = require('./tasks/assets');
const watch = require('./tasks/watch');

task('clean', function(done) {
  rimraf('./build', done);
});
task('build', series('clean', assets.copyAll, scripts.build));
task('develop', series('clean', watch.start));
