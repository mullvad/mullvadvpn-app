import log from 'electron-log';

before(() => {
  log.transports.console.level = false;
  log.transports.file.level = false;
});
