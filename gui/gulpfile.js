const fs = require('fs');
const { task, series, parallel } = require('gulp');

const scripts = require('./tasks/scripts');
const assets = require('./tasks/assets');
const watch = require('./tasks/watch');
const dist = require('./tasks/distribution');

task('clean', function (done) {
  fs.rmdir('./build', { recursive: true }, done);
});
task('build-proto', scripts.buildProto);
task('build', series('clean', parallel(assets.copyAll, scripts.buildProto), scripts.build));
task('develop', series('clean', scripts.buildProto, watch.start));
task('pack-win', series('build', dist.packWin));
task('pack-linux', series('build', dist.packLinux));
task('pack-mac', series('build', dist.packMac));
