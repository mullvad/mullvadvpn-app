// @flow

import { remote } from 'electron';
import log from 'electron-log';

import type { TunnelState } from './daemon-rpc';

export default class NotificationController {
  _activeNotification: ?Notification;

  notify(tunnelState: TunnelState) {
    switch (tunnelState) {
      case 'connecting':
        this._show('Connecting');
        break;
      case 'connected':
        this._show('Secured');
        break;
      case 'disconnected':
        this._show('Unsecured');
        break;
      case 'blocked':
        this._show('Blocked all connections');
        break;
      case 'disconnecting':
        // no-op
        break;
      default:
        log.error(`Unexpected TunnelStateTransition: ${(tunnelState: empty)}`);
    }
  }

  _show(message: string) {
    const lastNotification = this._activeNotification;
    const sameAsLastNotification = lastNotification && lastNotification.body === message;

    if (sameAsLastNotification || remote.getCurrentWindow().isVisible()) {
      return;
    }

    const newNotification = new Notification(remote.app.getName(), { body: message, silent: true });

    this._activeNotification = newNotification;

    newNotification.addEventListener('show', () => {
      // If the notification is closed too soon, it might still get shown. If that happens, close()
      // should be called again so that it is closed immediately.
      if (this._activeNotification !== newNotification) {
        newNotification.close();
      }
    });

    if (lastNotification) {
      lastNotification.close();
    }
  }
}
