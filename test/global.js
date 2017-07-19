import log from 'electron-log';
import { JSDOM } from 'jsdom';

before(() => {
  log.transports.console.level = false;
  log.transports.file.level = false;
});

beforeEach(() => {
  const dom = new JSDOM('<!doctype html><html><body></body></html>');
  const window = dom.window;
  global.window = window;
  global.document = window.document;
  global.navigator = window.navigator;
  global.HTMLInputElement = window.HTMLInputElement;
  global.Event = window.Event;
  global.MouseEvent = window.MouseEvent;
});
