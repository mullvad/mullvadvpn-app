const { parallel, series, src, dest } = require('gulp');
const envify = require('gulp-envify');
const ts = require('gulp-typescript');

const TscWatchClient = require('tsc-watch/client');

function makeWatchCompiler(onFirstSuccess) {
  const compileScripts = function() {
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
    .pipe(envify({ NODE_ENV: 'production' }))
    .pipe(dest('build'));
}

compileScripts.displayName = 'compile-scripts';

exports.build = compileScripts;
exports.makeWatchCompiler = makeWatchCompiler;
