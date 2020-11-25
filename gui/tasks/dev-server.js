const fs = require('fs');
const serverFactory = require('spa-server');

function startWebServer(done) {
  serverFactory.create({ path: './build', port: 8080 }).start();
  injectWorkingDirectory();
  done();
}

function injectWorkingDirectory() {
  const content = fs.readFileSync('build/src/renderer/index.js');
  fs.writeFileSync(
    'build/src/renderer/index.js',
    'try{process.chdir("build/src/renderer")}catch(e){}\n' + content,
  );
}

startWebServer.displayName = 'start-dev-server';

exports.start = startWebServer;
