const { app } = require('electron');
const log = require('electron-log');

log.transports.console.level = false;
log.transports.file.level = false;

// The default value has previously been false but will be changed to true in Electron 9. The
// false value has been deprecated in Electron 8. This can be removed when Electron 9 is used.
app.allowRendererProcessReuse = true;
