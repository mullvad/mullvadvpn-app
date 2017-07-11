import log from 'electron-log';
import jsdom from 'jsdom';

before(() => {
  global.document = jsdom.jsdom('<!doctype html><html><body></body></html>');
  global.window = document.defaultView;
  global.navigator = window.navigator;

  log.transports.console.level = false;
  log.transports.file.level = false;
});
