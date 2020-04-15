const { src, dest } = require('gulp');
const ts = require('gulp-typescript');
const inject = require('gulp-inject-string');
const TscWatchClient = require('tsc-watch/client');

function makeWatchCompiler(onFirstSuccess) {
  const compileScripts = function () {
    const watch = new TscWatchClient();
    watch.on('first_success', onFirstSuccess);
    watch.start('--noClear', '--sourceMap', '--project', '.');
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

compileScripts.displayName = 'compile-scripts';

exports.build = compileScripts;
exports.makeWatchCompiler = makeWatchCompiler;
