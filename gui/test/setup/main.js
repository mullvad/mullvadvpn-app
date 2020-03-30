const log = require('electron-log');
const { app } = require('electron');

log.transports.console.level = false;
log.transports.file.level = false;

app.allowRendererProcessReuse = true;
