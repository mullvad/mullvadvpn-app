const path = require('path');

const WORKSPACE_PROJECT_ROOT = path.resolve(__dirname, '..');

const BUILD_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'build');
const BUILD_STANDALONE_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'build-standalone');

const GEO_DIR = path.resolve('../../../dist-assets/geo');
const ICONS_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'assets/icons');
const IMAGES_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'assets/images');
const LOCALES_DIR = path.resolve(WORKSPACE_PROJECT_ROOT, 'locales');

exports.BUILD_DIR = BUILD_DIR;
exports.BUILD_STANDALONE_DIR = BUILD_STANDALONE_DIR;
exports.GEO_DIR = GEO_DIR;
exports.ICONS_DIR = ICONS_DIR;
exports.IMAGES_DIR = IMAGES_DIR;
exports.LOCALES_DIR = LOCALES_DIR;
exports.WORKSPACE_PROJECT_ROOT = WORKSPACE_PROJECT_ROOT;
