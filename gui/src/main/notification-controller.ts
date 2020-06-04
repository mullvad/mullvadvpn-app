import { app, nativeImage, NativeImage, Notification, shell } from 'electron';
import os from 'os';
import path from 'path';
import { sprintf } from 'sprintf-js';
import config from '../config.json';
import AccountExpiry from '../shared/account-expiry';
import { TunnelState, ITunnelStateRelayInfo } from '../shared/daemon-rpc-types';
import * as notifications from '../shared/notifications/notification';
import consumePromise from '../shared/promise';

export default class NotificationController {
  private lastTunnelStateAnnouncement?: { body: string; notification: Notification };
  private reconnecting = false;
  private presentedNotifications: { [key: string]: boolean } = {};
  private pendingNotifications: Notification[] = [];
  private notificationTitle = process.platform === 'linux' ? app.name : '';
  private notificationIcon?: NativeImage;

  constructor() {
    let usePngIcon;
    if (process.platform === 'linux') {
      usePngIcon = true;
    } else if (process.platform === 'win32') {
      usePngIcon = parseInt(os.release().split('.')[0], 10) >= 10;
    } else {
      usePngIcon = false;
    }

    if (usePngIcon) {
      const basePath = path.resolve(path.join(__dirname, '../../assets/images'));
      this.notificationIcon = nativeImage.createFromPath(
        path.join(basePath, 'icon-notification.png'),
      );
    }
  }

  public notifyTunnelState(
    evaluator: (
      notification: notifications.Notification<[TunnelState]>,
      tunnelState: TunnelState,
    ) => boolean,
    tunnelState: TunnelState,
  ) {
    let message: string | undefined;
    if (evaluator(notifications.connectingTo, tunnelState) && !this.reconnecting) {
      tunnelState = tunnelState as { state: 'connecting'; details?: ITunnelStateRelayInfo };
      message = sprintf(notifications.connectingTo.systemNotification.message, {
        location: tunnelState.details?.location?.hostname,
      });
    } else if (evaluator(notifications.connecting, tunnelState) && !this.reconnecting) {
      message = notifications.connecting.systemNotification.message;
    } else if (evaluator(notifications.connectedTo, tunnelState)) {
      tunnelState = tunnelState as { state: 'connected'; details: ITunnelStateRelayInfo };
      message = sprintf(notifications.connectedTo.systemNotification.message, {
        location: tunnelState.details?.location?.hostname,
      });
    } else if (evaluator(notifications.connected, tunnelState)) {
      message = notifications.connected.systemNotification.message;
    } else if (evaluator(notifications.disconnected, tunnelState)) {
      message = notifications.disconnected.systemNotification.message;
    } else if (evaluator(notifications.nonBlockingError, tunnelState)) {
      message = notifications.nonBlockingError.systemNotification.message;
    } else if (evaluator(notifications.error, tunnelState)) {
      message = notifications.error.systemNotification.message(tunnelState);
    } else if (evaluator(notifications.reconnecting, tunnelState)) {
      message = notifications.reconnecting.systemNotification.message;
      this.reconnecting = true;
    }

    this.reconnecting = false;
    if (message) {
      this.showTunnelStateNotification(message);
    }
  }

  public notifyInconsistentVersion() {
    this.presentNotificationOnce('inconsistent-version', () => {
      const notification = new Notification({
        title: this.notificationTitle,
        body: notifications.inconsistentVersion.systemNotification.message,
        silent: true,
        icon: this.notificationIcon,
      });
      this.scheduleNotification(notification);
    });
  }

  public notifyUnsupportedVersion(upgradeVersion: string) {
    this.presentNotificationOnce('unsupported-version', () => {
      const notification = new Notification({
        title: this.notificationTitle,
        body: sprintf(notifications.unsupportedVersion.systemNotification.message, {
          version: upgradeVersion,
        }),
        silent: true,
        icon: this.notificationIcon,
      });

      notification.on('click', () => {
        consumePromise(shell.openExternal(config.links.download));
      });

      this.scheduleNotification(notification);
    });
  }

  public closeToExpiryNotification(accountExpiry: AccountExpiry) {
    const duration = accountExpiry.durationUntilExpiry();
    const notification = new Notification({
      title: this.notificationTitle,
      body: sprintf(notifications.accountExpiry.systemNotification.message, {
        duration,
      }),
      silent: true,
      icon: this.notificationIcon,
    });
    this.scheduleNotification(notification);
  }

  public cancelPendingNotifications() {
    for (const notification of this.pendingNotifications) {
      notification.close();
    }
  }

  public resetTunnelStateAnnouncements() {
    this.lastTunnelStateAnnouncement = undefined;
  }

  private showTunnelStateNotification(message: string) {
    const lastAnnouncement = this.lastTunnelStateAnnouncement;
    const sameAsLastNotification = lastAnnouncement && lastAnnouncement.body === message;

    if (sameAsLastNotification) {
      return;
    }

    const newNotification = new Notification({
      title: this.notificationTitle,
      body: message,
      silent: true,
      icon: this.notificationIcon,
    });

    if (lastAnnouncement) {
      lastAnnouncement.notification.close();
    }

    this.lastTunnelStateAnnouncement = {
      body: message,
      notification: newNotification,
    };

    this.scheduleNotification(newNotification);
  }

  private presentNotificationOnce(notificationName: string, presentNotification: () => void) {
    const presented = this.presentedNotifications;
    if (!presented[notificationName]) {
      presented[notificationName] = true;
      presentNotification();
    }
  }

  private scheduleNotification(notification: Notification) {
    this.addPendingNotification(notification);

    notification.show();

    setTimeout(() => notification.close(), 4000);
  }

  private addPendingNotification(notification: Notification) {
    notification.on('close', () => {
      this.removePendingNotification(notification);
    });

    this.pendingNotifications.push(notification);
  }

  private removePendingNotification(notification: Notification) {
    const index = this.pendingNotifications.indexOf(notification);
    if (index !== -1) {
      this.pendingNotifications.splice(index, 1);
    }
  }
}
