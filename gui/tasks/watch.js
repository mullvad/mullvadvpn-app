const { parallel, series, watch } = require('gulp');
const electron = require('./electron');
const hotreload = require('./hotreload');
const devServer = require('./dev-server');
const assets = require('./assets');
const scripts = require('./scripts');

function watchMainScripts() {
  return watch(['build/src/main/**/*.js'], series(electron.stop, electron.start));
}

function watchRendererScripts() {
  return watch(['build/src/renderer/bundle.js'], series(hotreload.reload));
}

function watchCss() {
  return watch(['src/renderer/**/*.css'], series(assets.copyCss, hotreload.reload));
}

function watchConfig() {
  return watch(['src/config.json'], series(assets.copyConfig, hotreload.reload));
}

function watchHtml() {
  return watch(
    ['src/renderer/index.html'],
    series(assets.copyHtml, hotreload.inject, hotreload.reload),
  );
}

function watchStaticAssets() {
  return watch(['assets/**'], series(assets.copyStaticAssets, hotreload.reload));
}

watchMainScripts.displayName = 'watch-main-scripts';
watchRendererScripts.displayName = 'watch-renderer-scripts';
watchCss.displayName = 'watch-css';
watchConfig.displayName = 'watch-config';
watchHtml.displayName = 'watch-html';
watchStaticAssets.displayName = 'watch-static-assets';

exports.start = series(
  // copy all assets first
  assets.copyAll,

  // make an incremental script compiler running in watch mode
  scripts.makeWatchCompiler(
    // set up hotreload, run electron and begin watching filesystem for changes, after the first
    // successful build
    series(
      devServer.start,
      hotreload.start,
      electron.start,
      parallel(
        watchMainScripts,
        watchRendererScripts,
        watchCss,
        watchConfig,
        watchHtml,
        watchStaticAssets,
      ),
    ),
  ),
);
