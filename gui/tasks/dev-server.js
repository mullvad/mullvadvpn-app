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
      let s = data.toString();

      // Add code that changes to the correct working directory after `"use strict";` which is
      // located on the first line of the source file.
      const index = s.indexOf('\n');

      if (index !== -1) {
        const insertionIndex = index + 1;
        s =
          s.slice(0, insertionIndex) +
          'try{process.chdir("build/src/renderer")}catch(e){}\n' +
          s.slice(insertionIndex);
      }

      write(s);
    };
  }

  next();
}

startWebServer.displayName = 'start-dev-server';

exports.start = startWebServer;
