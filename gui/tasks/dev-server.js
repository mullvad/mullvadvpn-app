const serverFactory = require('spa-server');

function startWebServer(done) {
  serverFactory
    .create({
      path: './build',
      port: 8080,
      middleware: [correctWorkingDirectory],
    })
    .start(done);
}

function correctWorkingDirectory(request, response, next) {
  if (request.url === '/src/renderer/index.js') {
    const write = response.write.bind(response);
    response.write = (data) => {
      // Add code that changes to the correct working directory after `"use strict";`
      const lines = data.toString().split('\n');
      lines.splice(1, 0, 'try{process.chdir("build/src/renderer")}catch(e){}');
      write(lines.join('\n'));
    };
  }

  next();
}

startWebServer.displayName = 'start-dev-server';

exports.start = startWebServer;
