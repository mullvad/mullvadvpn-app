const fs = require('fs');
const { execSync } = require('child_process');
const { task, series } = require('gulp');

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
task('set-test-env', function (done) {
  process.env.NODE_ENV = 'test';
  done();
});

task('clean', function (done) {
  fs.rm('./build', { recursive: true, force: true }, done);
});
task('clean-standalone', function (done) {
  fs.rm('./build-standalone', { recursive: true, force: true }, done);
});
task('build-vite-prod', function (done) {
  execSync('npm run build:vite');
  done();
});
task('build-standalone-transpile', function (done) {
  execSync('npx tsc -p tsconfig.standalone.json');
  done();
});
task('build-proto', scripts.buildProto);
task(
  'develop',
  series(
    'clean',
    'set-dev-env',
    scripts.buildNseventforwarder,
    scripts.buildWindowsUtils,
    watch.start,
  ),
);
task('build', series('clean', 'set-prod-env', assets.copyAll, scripts.build));
task('build-vite', series('clean', 'set-prod-env', 'build-vite-prod', assets.copyAllVite));
task('build-test', series('clean', 'set-test-env', 'build-vite-prod', assets.copyAllVite));
task('build-standalone', series('clean-standalone', 'set-prod-env', 'build-standalone-transpile'));
task('pack-win', series('build-vite', dist.packWin));
task('pack-linux', series('build-vite', dist.packLinux));
task('pack-mac', series('build-vite', dist.packMac));
