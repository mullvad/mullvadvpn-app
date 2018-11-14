// @flow

import { remote } from 'electron';
import log from 'electron-log';
import config from '../../config';

import type { TunnelStateTransition } from './daemon-rpc-proxy';

export default class NotificationController {
  _lastTunnelStateNotification: ?Notification;
  _reconnecting = false;
  _presentedNotifications = {};
  _pendingNotifications: Array<Notification> = [];

  notifyTunnelState(tunnelState: TunnelStateTransition) {
    switch (tunnelState.state) {
      case 'connecting':
        if (!this._reconnecting) {
          this._showTunnelStateNotification('Connecting');
        }
        break;
      case 'connected':
        this._showTunnelStateNotification('Secured');
        break;
      case 'disconnected':
        this._showTunnelStateNotification('Unsecured');
        break;
      case 'blocked':
        this._showTunnelStateNotification('Blocked all connections');
        break;
      case 'disconnecting':
        switch (tunnelState.details) {
          case 'nothing':
          case 'block':
            // no-op
            break;
          case 'reconnect':
            this._showTunnelStateNotification('Reconnecting');
            this._reconnecting = true;
            return;
        }
        break;
      default:
        log.error(`Unexpected TunnelStateTransition: ${(tunnelState.state: empty)}`);
    }

    this._reconnecting = false;
  }

  notifyInconsistentVersion() {
    if (remote.getCurrentWindow().isVisible()) {
      return;
    }

    this._presentNotificationOnce('inconsistent-version', () => {
      const notification = new Notification(remote.app.getName(), {
        body: 'Inconsistent internal version information, please restart the app',
        silent: true,
      });
      this._addPendingNotification(notification);
    });
  }

  notifyUnsupportedVersion(upgradeVersion: string) {
    if (remote.getCurrentWindow().isVisible()) {
      return;
    }

    this._presentNotificationOnce('unsupported-version', () => {
      const notification = new Notification(remote.app.getName(), {
        body: `You are running an unsupported app version. Please upgrade to ${upgradeVersion} now to ensure your security`,
        silent: true,
      });

      notification.addEventListener('click', () => {
        remote.shell.openExternal(config.links.download);
      });

      this._addPendingNotification(notification);
    });
  }

  cancelPendingNotifications() {
    for (const notification of this._pendingNotifications) {
      this._closeNotification(notification);
    }
  }

  _showTunnelStateNotification(message: string) {
    const lastNotification = this._lastTunnelStateNotification;
    const sameAsLastNotification = lastNotification && lastNotification.body === message;

    if (sameAsLastNotification || remote.getCurrentWindow().isVisible()) {
      return;
    }

    const newNotification = new Notification(remote.app.getName(), {
      body: message,
      silent: true,
    });

    if (lastNotification) {
      this._closeNotification(lastNotification);
    }

    this._lastTunnelStateNotification = newNotification;
    this._addPendingNotification(newNotification);
  }

  _presentNotificationOnce(notificationName: string, presentNotification: () => void) {
    const presented = this._presentedNotifications;
    if (!presented[notificationName]) {
      presented[notificationName] = true;
      presentNotification();
    }
  }

  _closeNotification(notification: Notification) {
    // If the notification is closed too soon, it might still get shown. If that happens, close()
    // should be called again so that it is closed immediately.
    // Tracking issue: https://github.com/electron/electron/issues/12887
    notification.addEventListener('show', () => {
      notification.close();
    });

    notification.close();
  }

  _addPendingNotification(notification: Notification) {
    // Quirk: chromium postpones the 'close' event until new notifications pump the queue or window
    // becomes visible. It's possible that there is going to be one stale notification in
    // `_pendingNotifications` array but that shouldn't be a big deal.
    notification.addEventListener('close', () => {
      this._removePendingNotification(notification);
    });

    this._pendingNotifications.push(notification);
  }

  _removePendingNotification(notification: Notification) {
    const index = this._pendingNotifications.indexOf(notification);
    if (index !== -1) {
      this._pendingNotifications.splice(index, 1);
    }
  }
}
