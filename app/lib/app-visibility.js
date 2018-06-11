// @flow
import { ipcRenderer } from 'electron';

type EventHandler = (boolean) => any;

export default class AppVisiblityObserver {
  _handler: EventHandler;

  constructor(handler: EventHandler) {
    this._handler = handler;

    ipcRenderer.on('show-window', this._handleShowEvent).on('hide-window', this._handleHideEvent);
  }

  dispose() {
    ipcRenderer
      .removeListener('show-window', this._handleShowEvent)
      .removeListener('hide-window', this._handleHideEvent);
  }

  _handleShowEvent = (_event) => {
    this._handler(true);
  };

  _handleHideEvent = (_event) => {
    this._handler(false);
  };
}
