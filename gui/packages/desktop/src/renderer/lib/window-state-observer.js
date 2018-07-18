// @flow

import { remote } from 'electron';

type EventListener = () => void;

// Tiny helper for detecting the window state.
export default class WindowStateObserver {
  _onShow: ?EventListener;
  _onHide: ?EventListener;

  get onShow() {
    return this._onShow;
  }

  set onShow(listener: ?EventListener) {
    const currentWindow = remote.getCurrentWindow();
    const oldListener = this._onShow;
    if (oldListener) {
      currentWindow.removeListener('show', oldListener);
    }

    if (listener) {
      currentWindow.addListener('show', listener);
    }

    this._onShow = listener;
  }

  get onHide() {
    return this._onHide;
  }

  set onHide(listener: ?EventListener) {
    const currentWindow = remote.getCurrentWindow();
    const oldListener = this._onHide;
    if (oldListener) {
      currentWindow.removeListener('hide', oldListener);
    }

    if (listener) {
      currentWindow.addListener('hide', listener);
    }

    this._onHide = listener;
  }

  constructor() {
    // Because BrowserWindow persists between page reloads,
    // it's important to release event handlers when that happens.
    window.addEventListener('beforeunload', this._onBeforeUnload);
  }

  _onBeforeUnload = () => {
    this.dispose();
  };

  dispose() {
    this.onShow = null;
    this.onHide = null;

    window.removeEventListener('beforeunload', this._onBeforeUnload);
  }
}
