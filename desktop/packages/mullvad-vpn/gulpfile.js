const fs = require('fs');
const { task, series, parallel } = require('gulp');

const scripts = require('./tasks/scripts');
const assets = require('./tasks/assets');
const watch = require('./tasks/watch');
const dist = require('./tasks/distribution');

task('set-dev-env', function (done) {
  process.env.NODE_ENV = 'development';
  done();
});
task('set-prod-env', function (done) {
  process.env.NODE_ENV = 'production';
  done();
});

task('clean', function (done) {
  fs.rm('./build', { recursive: true, force: true }, done);
});
task('build-proto', scripts.buildProto);
task(
  'build',
  series('clean', 'set-prod-env', parallel(assets.copyAll, scripts.buildProto), scripts.build),
);
task(
  'develop',
  series(
    'clean',
    'set-dev-env',
    scripts.buildProto,
    scripts.buildNseventforwarder,
    scripts.buildWinShortcuts,
    watch.start,
  ),
);
task('pack-win', series('build', dist.packWin));
task('pack-linux', series('build', dist.packLinux));
task('pack-mac', series('build', dist.packMac));
