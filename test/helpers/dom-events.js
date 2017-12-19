// @flow
const keycodes = {
  '1': { which: 49, keyCode: 49 },
  '2': { which: 50, keyCode: 50 },
  '3': { which: 51, keyCode: 51 },
  '4': { which: 52, keyCode: 52 },
  '5': { which: 53, keyCode: 53 },
  '6': { which: 54, keyCode: 54 },
  '7': { which: 55, keyCode: 55 },
  '8': { which: 56, keyCode: 56 },
  '9': { which: 57, keyCode: 57 },
  '0': { which: 48, keyCode: 48 },
  Tab: { which: 9, keyCode: 9 },
  Enter: { which: 13, keyCode: 13 },
  Backspace: { which: 8, keyCode: 8 }
};

export type Keycode = $Keys<typeof keycodes>;

export function createKeyEvent(key: Keycode): Object {
  return Object.assign({}, { key }, keycodes[key]);
}
