import log from 'electron-log';
import jsdom from 'jsdom';

before(() => {
  log.transports.console.level = false;
  log.transports.file.level = false;
});

beforeEach(() => {
  global.document = jsdom.jsdom('<!doctype html><html><body></body></html>');
  global.window = document.defaultView;
  global.navigator = window.navigator;
  global.HTMLInputElement = window.HTMLInputElement;
  global.MouseEvent = window.MouseEvent;
});
