const { exec } = require('child_process');
const { dest, series, parallel } = require('gulp');
const ts = require('gulp-typescript');
const inject = require('gulp-inject-string');
const sourcemaps = require('gulp-sourcemaps');
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
        makeBrowserifyRenderer(true),
        makeBrowserifyPreload(true),
      )(() => {
        if (firstBuild) {
          firstBuild = false;
          onFirstSuccess();
        }
      }),
    );
    watch.start('--noClear', '--sourceMap', '--inlineSources', '--incremental', '--project', '.');
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

function makeBrowserifyRenderer(debug) {
  const browserifyRenderer = () => {
    let stream = browserify({ entries: './build/src/renderer/index.js', debug })
      .bundle()
      .pipe(source('bundle.js'))
      .pipe(buffer());

    if (debug) {
      stream = stream.pipe(sourcemaps.init({ loadMaps: true })).pipe(sourcemaps.write());
    }

    return stream.pipe(dest('./build/src/renderer/'));
  };

  browserifyRenderer.displayName = 'browserify-renderer';
  return browserifyRenderer;
}

function makeBrowserifyPreload(debug) {
  const browserifyPreload = () => {
    let stream = browserify({
      entries: './build/src/renderer/preload.js',
      debug,
    })
      .exclude('electron')
      .bundle()
      .pipe(source('preloadBundle.js'))
      .pipe(buffer());

    if (debug) {
      stream = stream.pipe(sourcemaps.init({ loadMaps: true })).pipe(sourcemaps.write());
    }

    return stream.pipe(dest('./build/src/renderer/'));
  };

  browserifyPreload.displayName = 'browserify-preload';
  return browserifyPreload;
}

function buildProto(callback) {
  exec('bash ./scripts/build-proto.sh', (err) => callback(err));
}

compileScripts.displayName = 'compile-scripts';
buildProto.displayName = 'build-proto';

exports.build = series(
  compileScripts,
  parallel(makeBrowserifyPreload(false), makeBrowserifyRenderer(false)),
);
exports.buildProto = buildProto;
exports.makeWatchCompiler = makeWatchCompiler;
