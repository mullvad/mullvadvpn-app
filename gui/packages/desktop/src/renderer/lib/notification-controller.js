// @flow

import { remote } from 'electron';
import log from 'electron-log';
import config from '../../config';

import type { TunnelStateTransition } from './daemon-rpc';

export default class NotificationController {
  _activeNotification: ?Notification;
  _reconnecting = false;
  _presentedNotifications = {};

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
      new Notification(remote.app.getName(), {
        body: 'Inconsistent internal version information, please restart the app',
        silent: true,
      });
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
    });
  }

  _showTunnelStateNotification(message: string) {
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
      // Tracking issue: https://github.com/electron/electron/issues/12887
      if (this._activeNotification !== newNotification) {
        newNotification.close();
      }
    });

    if (lastNotification) {
      lastNotification.close();
    }
  }

  _presentNotificationOnce(notificationName: string, presentNotification: () => void) {
    const presented = this._presentedNotifications;
    if (!presented[notificationName]) {
      presented[notificationName] = true;
      presentNotification();
    }
  }
}
