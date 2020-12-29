const { exec } = require('child_process');
const { src, dest, series } = require('gulp');
const ts = require('gulp-typescript');
const inject = require('gulp-inject-string');
const TscWatchClient = require('tsc-watch/client');
const browserify = require('browserify');
const buffer = require('vinyl-buffer');
const source = require('vinyl-source-stream');

function makeWatchCompiler(onFirstSuccess) {
  const compileScripts = function () {
    const watch = new TscWatchClient();
    watch.on('first_success', onFirstSuccess);
    watch.on('success', browserifyRenderer);
    watch.start('--noClear', '--inlineSourceMap', '--incremental', '--project', '.');
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

function browserifyRenderer() {
  return browserify({ entries: './build/src/renderer/index.js' })
    .bundle()
    .pipe(source('bundle.js'))
    .pipe(buffer())
    .pipe(dest('./build/src/renderer/'));
}

function buildProto(callback) {
  exec('bash ./scripts/build-proto.sh', (err) => callback(err));
}

compileScripts.displayName = 'compile-scripts';
browserifyRenderer.displayName = 'browserify-renderer';
buildProto.displayName = 'build-proto';

exports.build = series(compileScripts, browserifyRenderer);
exports.buildProto = buildProto;
exports.makeWatchCompiler = makeWatchCompiler;
