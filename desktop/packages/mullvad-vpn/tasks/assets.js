const { parallel, src, dest } = require('gulp');

function copyStaticAssets() {
  return src('assets/**').pipe(dest('build/assets'));
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

function copyGeoData() {
  return src('../../../dist-assets/geo/*.gl').pipe(dest('build/assets/geo'));
}

copyStaticAssets.displayName = 'copy-static-assets';
copyCss.displayName = 'copy-css';
copyHtml.displayName = 'copy-html';
copyLocales.displayName = 'copy-locales';
copyGeoData.displayName = 'copy-geo-data';

exports.copyAll = parallel(copyStaticAssets, copyCss, copyHtml, copyLocales, copyGeoData);
exports.copyStaticAssets = copyStaticAssets;
exports.copyCss = copyCss;
exports.copyHtml = copyHtml;
exports.copyGeoData = copyGeoData;
