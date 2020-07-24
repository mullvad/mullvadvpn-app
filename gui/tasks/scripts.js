const { exec } = require('child_process');
const { src, dest, series } = require('gulp');
const ts = require('gulp-typescript');
const inject = require('gulp-inject-string');
const TscWatchClient = require('tsc-watch/client');

function makeWatchCompiler(onFirstSuccess) {
  const compileScripts = function () {
    const watch = new TscWatchClient();
    watch.on('first_success', onFirstSuccess);
    watch.start('--noClear', '--sourceMap', '--incremental', '--project', '.');
    return watch.tsc;
  };
  compileScripts.displayName = 'compile-scripts-watch';

  return compileScripts;
}

function compileScripts() {
  const tsProject = ts.createProject('tsconfig.json');

  return tsProject
    .src()
    .pipe(tsProject())
    .pipe(inject.replace('process.env.NODE_ENV', '"production"'))
    .pipe(dest('build'));
}

function buildProto(callback) {
  exec('./scripts/build-proto.sh', () => callback());
}

compileScripts.displayName = 'compile-scripts';
buildProto.displayName = 'build-proto';

exports.build = compileScripts;
exports.buildProto = buildProto;
exports.makeWatchCompiler = makeWatchCompiler;
