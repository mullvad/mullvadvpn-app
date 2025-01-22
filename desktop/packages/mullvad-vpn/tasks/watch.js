const { parallel, series, watch } = require('gulp');
const electron = require('./electron');
const assets = require('./assets');
const scripts = require('./scripts');

function watchMainScripts() {
  return watch(['build/src/main/**/*.js'], series(electron.reloadMain));
}

function watchCss() {
  return watch(['src/renderer/**/*.css'], series(assets.copyCss, electron.reloadRenderer));
}

function watchHtml() {
  return watch(['src/renderer/index.html'], series(assets.copyHtml, electron.reloadRenderer));
}

function watchStaticAssets() {
  return watch(
    ['assets/**', '../dist-assets/geo/*.gl'],
    series(assets.copyStaticAssets, assets.copyGeoData, electron.reloadRenderer),
  );
}

watchMainScripts.displayName = 'watch-main-scripts';
watchCss.displayName = 'watch-css';
watchHtml.displayName = 'watch-html';
watchStaticAssets.displayName = 'watch-static-assets';

exports.start = series(
  // copy all assets first
  assets.copyAll,

  // make an incremental script compiler running in watch mode
  scripts.makeWatchCompiler(
    // set up hotreload, run electron and begin watching filesystem for changes, after the first
    // successful build
    series(electron.start, parallel(watchMainScripts, watchCss, watchHtml, watchStaticAssets)),
    electron.reloadRenderer,
  ),
);
