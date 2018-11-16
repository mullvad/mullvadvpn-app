// @flow

import { shell, Notification } from 'electron';
import log from 'electron-log';
import config from '../config';

import type { TunnelStateTransition } from './daemon-rpc';

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
    this._presentNotificationOnce('inconsistent-version', () => {
      const notification = new Notification({
        body: 'Inconsistent internal version information, please restart the app',
        silent: true,
      });
      this._scheduleNotification(notification);
    });
  }

  notifyUnsupportedVersion(upgradeVersion: string) {
    this._presentNotificationOnce('unsupported-version', () => {
      const notification = new Notification({
        body: `You are running an unsupported app version. Please upgrade to ${upgradeVersion} now to ensure your security`,
        silent: true,
      });

      notification.on('click', () => {
        shell.openExternal(config.links.download);
      });

      this._scheduleNotification(notification);
    });
  }

  cancelPendingNotifications() {
    for (const notification of this._pendingNotifications) {
      notification.close();
    }
  }

  _showTunnelStateNotification(message: string) {
    const lastNotification = this._lastTunnelStateNotification;
    const sameAsLastNotification = lastNotification && lastNotification.body === message;

    if (sameAsLastNotification) {
      return;
    }

    const newNotification = new Notification({
      body: message,
      silent: true,
    });

    if (lastNotification) {
      lastNotification.close();
    }

    this._lastTunnelStateNotification = newNotification;
    this._scheduleNotification(newNotification);
  }

  _presentNotificationOnce(notificationName: string, presentNotification: () => void) {
    const presented = this._presentedNotifications;
    if (!presented[notificationName]) {
      presented[notificationName] = true;
      presentNotification();
    }
  }

  _scheduleNotification(notification: Notification) {
    this._addPendingNotification(notification);

    notification.show();
  }

  _addPendingNotification(notification: Notification) {
    notification.on('close', () => {
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
