import jsdom from 'jsdom';

global.document = jsdom.jsdom('<!doctype html><html><body></body></html>');
global.window = document.defaultView;
global.navigator = window.navigator;

const keyMap = {
  _1: { key: '1', which: 49, keyCode: 49 },
  _2: { key: '2', which: 50, keyCode: 50 },
  _3: { key: '3', which: 51, keyCode: 51 },
  _4: { key: '4', which: 52, keyCode: 52 },
  _5: { key: '5', which: 53, keyCode: 53 },
  _6: { key: '6', which: 54, keyCode: 54 },
  _7: { key: '7', which: 55, keyCode: 55 },
  _8: { key: '8', which: 56, keyCode: 56 },
  _9: { key: '9', which: 57, keyCode: 57 },
  _0: { key: '0', which: 48, keyCode: 48 },
  Tab: { which: 9, keyCode: 9 },
  Enter: { which: 13, keyCode: 13 },
  Backspace: { which: 8, keyCode: 8 }
};

export const KeyType = (() => {
  let dict = {};
  for(const i of Object.keys(keyMap)) {
    dict[i] = i;
  }

  return dict;
})();

export function createKeyEvent(key) {
  return Object.assign({ key }, keyMap[key]);
}