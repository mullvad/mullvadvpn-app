const { exec } = require('child_process');
const { dest, series, parallel } = require('gulp');
const ts = require('gulp-typescript');
const inject = require('gulp-inject-string');
const TscWatchClient = require('tsc-watch/client');
const browserify = require('browserify');
const buffer = require('vinyl-buffer');
const source = require('vinyl-source-stream');

function makeWatchCompiler(onFirstSuccess) {
  let firstBuild = true;

  const compileScripts = function () {
    const watch = new TscWatchClient();
    watch.on('success', () =>
      parallel(
        browserifyRenderer,
        browserifyPreload,
      )(() => {
        if (firstBuild) {
          firstBuild = false;
          onFirstSuccess();
        }
      }),
    );
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

function browserifyPreload() {
  return browserify({
    entries: './build/src/renderer/preload.js',
  })
    .exclude('fs')
    .exclude('electron')
    .bundle()
    .pipe(source('preloadBundle.js'))
    .pipe(buffer())
    .pipe(dest('./build/src/renderer/'));
}

function buildProto(callback) {
  exec('bash ./scripts/build-proto.sh', (err) => callback(err));
}

compileScripts.displayName = 'compile-scripts';
browserifyRenderer.displayName = 'browserify-renderer';
browserifyPreload.displayName = 'browserify-preload';
buildProto.displayName = 'build-proto';

exports.build = series(compileScripts, parallel(browserifyPreload, browserifyRenderer));
exports.buildProto = buildProto;
exports.makeWatchCompiler = makeWatchCompiler;
