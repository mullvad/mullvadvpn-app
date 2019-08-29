const { parallel, src, dest } = require('gulp');

function copyStaticAssets() {
  return src('assets/**').pipe(dest('build/assets'));
}

function copyConfig() {
  return src('src/config.json').pipe(dest('build/src'));
}

function copyCss() {
  return src('src/renderer/**/*.css').pipe(dest('build/src/renderer'));
}

function copyHtml() {
  return src('src/renderer/index.html').pipe(dest('build/src/renderer'));
}

function copyLocales() {
  return src('locales/**/*.po').pipe(dest('build/locales'));
}

copyStaticAssets.displayName = 'copy-static-assets';
copyConfig.displayName = 'copy-config';
copyCss.displayName = 'copy-css';
copyHtml.displayName = 'copy-html';
copyLocales.displayName = 'copy-locales';

exports.copyAll = parallel(copyStaticAssets, copyConfig, copyCss, copyHtml, copyLocales);
exports.copyStaticAssets = copyStaticAssets;
exports.copyCss = copyCss;
exports.copyHtml = copyHtml;
exports.copyConfig = copyConfig;
