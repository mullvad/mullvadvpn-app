const { exec } = require('child_process');
const fs = require('fs');
const { dest, series, parallel } = require('gulp');
const ts = require('gulp-typescript');
const inject = require('gulp-inject-string');
const sourcemaps = require('gulp-sourcemaps');
const TscWatchClient = require('tsc-watch/client');
const browserify = require('browserify');
const buffer = require('vinyl-buffer');
const source = require('vinyl-source-stream');

function makeWatchCompiler(onFirstSuccess, onSuccess) {
  let firstBuild = true;
  let lastBundle;
  let lastPreloadBundle;

  const compileScripts = function () {
    const watch = new TscWatchClient();
    watch.on('success', () =>
      parallel(
        makeBrowserifyRenderer(true),
        makeBrowserifyPreload(true),
      )(async () => {
        const wasFirstBuild = firstBuild;
        if (firstBuild) {
          firstBuild = false;
          onFirstSuccess();
        }

        let bundle = await fs.promises.readFile('./build/src/renderer/bundle.js');
        let preloadBundle = await fs.promises.readFile('./build/src/renderer/preloadBundle.js');
        if (
          !lastBundle ||
          !preloadBundle ||
          !lastBundle.equals(bundle) ||
          !lastPreloadBundle.equals(preloadBundle)
        ) {
          lastBundle = bundle;
          lastPreloadBundle = preloadBundle;
          if (!wasFirstBuild) {
            onSuccess();
          }
        }
      }),
    );
    watch.start(
      '--noClear',
      '--sourceMap',
      '--inlineSources',
      '--incremental',
      '--project',
      './tsconfig.dev.json',
    );
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
      detectGlobals: false,
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

function buildNseventforwarder(callback) {
  if (process.platform === 'darwin') {
    exec('npm -w nseventforwarder run build-debug', (err) => callback(err));
  } else {
    callback();
  }
}

function buildWindowsUtils(callback) {
  if (process.platform === 'win32') {
    exec('npm -w windows-utils run build-debug', (err) => callback(err));
  } else {
    callback();
  }
}

compileScripts.displayName = 'compile-scripts';
buildNseventforwarder.displayName = 'build-nseventforwarder';
buildWindowsUtils.displayName = 'build-windows-utils';

exports.build = series(
  compileScripts,
  parallel(makeBrowserifyPreload(false), makeBrowserifyRenderer(false)),
);
exports.buildNseventforwarder = buildNseventforwarder;
exports.buildWindowsUtils = buildWindowsUtils;
exports.makeWatchCompiler = makeWatchCompiler;
