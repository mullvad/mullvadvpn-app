import { shell, Notification } from 'electron';
import config from '../config.json';

import { TunnelStateTransition } from '../shared/daemon-rpc-types';

export default class NotificationController {
  _lastTunnelStateAnnouncement?: { body: string; notification: Notification };
  _reconnecting = false;
  _presentedNotifications: { [key: string]: boolean } = {};
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
        switch (tunnelState.details.reason) {
          case 'set_firewall_policy_error':
            this._showTunnelStateNotification('Critical failure - Unsecured');
            break;
          default:
            this._showTunnelStateNotification('Blocked all connections');
            break;
        }
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
    }

    this._reconnecting = false;
  }

  notifyInconsistentVersion() {
    this._presentNotificationOnce('inconsistent-version', () => {
      const notification = new Notification({
        title: '',
        body: 'Inconsistent internal version information, please restart the app',
        silent: true,
      });
      this._scheduleNotification(notification);
    });
  }

  notifyUnsupportedVersion(upgradeVersion: string) {
    this._presentNotificationOnce('unsupported-version', () => {
      const notification = new Notification({
        title: '',
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
    const lastAnnouncement = this._lastTunnelStateAnnouncement;
    const sameAsLastNotification = lastAnnouncement && lastAnnouncement.body === message;

    if (sameAsLastNotification) {
      return;
    }

    const newNotification = new Notification({
      title: '',
      body: message,
      silent: true,
    });

    if (lastAnnouncement) {
      lastAnnouncement.notification.close();
    }

    this._lastTunnelStateAnnouncement = {
      body: message,
      notification: newNotification,
    };

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

    setTimeout(() => notification.close(), 4000);
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
