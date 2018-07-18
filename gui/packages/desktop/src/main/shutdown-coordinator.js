// @flow

import log from 'electron-log';
import { app, ipcMain } from 'electron';
import type { WebContents } from 'electron';

// The timeout before the shutdown is enforced
const SHUTDOWN_TIMEOUT = 3000;

/**
 * Shutdown coordinator postpones the application shutdown until ShutdownHandler finished the
 * shutdown sequence.
 *
 * ShutdownCoordinator can only be created from main process.
 */
export default class ShutdownCoordinator {
  _canQuit = false;
  _isQuitting = false;
  _webContents: WebContents;
  _shutdownTimeout: ?TimeoutID;

  constructor(webContents: WebContents) {
    this._webContents = webContents;

    app.on('before-quit', this._onBeforeQuit);
    ipcMain.on('app-shutdown-reply', this._onShutdownReply);
  }

  dispose() {
    app
      .removeListener('before-quit', this._onBeforeQuit)
      .removeListener('app-shutdown-reply', this._onShutdownReply);
  }

  _onBeforeQuit = (event) => {
    if (!this._canQuit) {
      // make sure we don't call the shutdown handler twice
      if (!this._isQuitting) {
        this._isQuitting = true;

        // start timer to force shutdown if the renderer process is not able to handle shutdown in
        // a timely manner.
        this._shutdownTimeout = setTimeout(this._onShutdownTimeout, SHUTDOWN_TIMEOUT);

        this._webContents.send('app-shutdown');
      }

      event.preventDefault();
    }
  };

  _onShutdownReply = () => {
    const shutdownTimeout = this._shutdownTimeout;
    if (shutdownTimeout) {
      clearTimeout(shutdownTimeout);
    }
    this._grantShutdown();
  };

  _onShutdownTimeout = () => {
    log.warn('It took longer than expected to shutdown. Forcing shutdown now.');
    this._grantShutdown();
  };

  _grantShutdown() {
    this._canQuit = true;
    app.quit();
  }
}
