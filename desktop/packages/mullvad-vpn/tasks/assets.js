const { parallel, src, dest } = require('gulp');

function copyStaticAssets() {
  return src('assets/**').pipe(dest('build/assets'));
}

function copyImagesVite() {
  return src('assets/images/**').pipe(dest('build/assets/images'));
}

function copyIconsVite() {
  return src('assets/icons/**').pipe(dest('build/assets/icons'));
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

function copyLocalesVite() {
  return src('locales/**/*.po').pipe(dest('build/locales'));
}

function copyGeoData() {
  return src('../../../dist-assets/geo/*.gl').pipe(dest('build/assets/geo'));
}
function copyGeoDataVite() {
  return src('../../../dist-assets/geo/*.gl').pipe(dest('build/assets/geo'));
}

copyStaticAssets.displayName = 'copy-static-assets';
copyCss.displayName = 'copy-css';
copyHtml.displayName = 'copy-html';
copyLocales.displayName = 'copy-locales';
copyGeoData.displayName = 'copy-geo-data';

exports.copyAll = parallel(copyStaticAssets, copyCss, copyHtml, copyLocales, copyGeoData);
exports.copyAllVite = parallel(copyImagesVite, copyIconsVite, copyLocalesVite, copyGeoDataVite);
exports.copyStaticAssets = copyStaticAssets;
exports.copyCss = copyCss;
exports.copyHtml = copyHtml;
exports.copyGeoData = copyGeoData;
