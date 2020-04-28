import { app, nativeImage, NativeImage, Notification, shell } from 'electron';
import os from 'os';
import path from 'path';
import { sprintf } from 'sprintf-js';
import config from '../config.json';
import AccountExpiry from '../shared/account-expiry';
import { TunnelState } from '../shared/daemon-rpc-types';
import { messages } from '../shared/gettext';
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

  public notifyTunnelState(tunnelState: TunnelState) {
    switch (tunnelState.state) {
      case 'connecting':
        if (!this.reconnecting) {
          const details = tunnelState.details;
          if (details && details.location && details.location.hostname) {
            const msg = sprintf(
              // TRANSLATORS: The message showed when a server is being connected to.
              // TRANSLATORS: Available placeholder:
              // TRANSLATORS: %(location) - name of the server location we're connecting to (e.g. "se-got-003")
              messages.pgettext('notifications', 'Connecting to %(location)s'),
              {
                location: details.location.hostname,
              },
            );
            this.showTunnelStateNotification(msg);
          } else {
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Connecting'));
          }
        }
        break;
      case 'connected':
        {
          const details = tunnelState.details;
          if (details.location && details.location.hostname) {
            const msg = sprintf(
              // TRANSLATORS: The message showed when a server has been connected to.
              // TRANSLATORS: Available placeholder:
              // TRANSLATORS: %(location) - name of the server location we're connected to (e.g. "se-got-003")
              messages.pgettext('notifications', 'Connected to %(location)s'),
              {
                location: details.location.hostname,
              },
            );
            this.showTunnelStateNotification(msg);
          } else {
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Secured'));
          }
        }
        break;
      case 'disconnected':
        this.showTunnelStateNotification(messages.pgettext('notifications', 'Unsecured'));
        break;
      case 'error':
        if (tunnelState.details.isBlocking) {
          if (
            tunnelState.details.cause.reason === 'tunnel_parameter_error' &&
            tunnelState.details.cause.details === 'no_wireguard_key'
          ) {
            this.showTunnelStateNotification(
              messages.pgettext(
                'notifications',
                'Blocking internet: Valid WireGuard key is missing',
              ),
            );
          } else {
            this.showTunnelStateNotification(
              messages.pgettext('notifications', 'Blocking internet'),
            );
          }
        } else {
          this.showTunnelStateNotification(
            messages.pgettext('notifications', 'Critical error (your attention is required)'),
          );
        }
        break;
      case 'disconnecting':
        switch (tunnelState.details) {
          case 'nothing':
          case 'block':
            // no-op
            break;
          case 'reconnect':
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Reconnecting'));
            this.reconnecting = true;
            return;
        }
        break;
    }

    this.reconnecting = false;
  }

  public notifyInconsistentVersion() {
    this.presentNotificationOnce('inconsistent-version', () => {
      const notification = new Notification({
        title: this.notificationTitle,
        body: messages.pgettext(
          'notifications',
          'Inconsistent internal version information, please restart the app',
        ),
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
        body: sprintf(
          // TRANSLATORS: The system notification displayed to the user when the running app becomes unsupported.
          // TRANSLATORS: Available placeholder:
          // TRANSLATORS: %(version) - the newest available version of the app
          messages.pgettext(
            'notifications',
            'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
          ),
          {
            version: upgradeVersion,
          },
        ),
        silent: true,
        icon: this.notificationIcon,
      });

      notification.on('click', () => {
        consumePromise(shell.openExternal(config.links.download));
      });

      this.scheduleNotification(notification);
    });
  }

  public notifyKeyGenerationFailed() {
    const notification = new Notification({
      title: this.notificationTitle,
      body: messages.pgettext('notifications', 'Wireguard key generation failed'),
      silent: true,
      icon: this.notificationIcon,
    });
    this.scheduleNotification(notification);
  }

  public closeToExpiryNotification(accountExpiry: AccountExpiry) {
    const duration = accountExpiry.durationUntilExpiry();
    const notification = new Notification({
      title: this.notificationTitle,
      body: sprintf(
        // TRANSLATORS: The system notification displayed to the user when the account credit is close to expiry.
        // TRANSLATORS: Available placeholder:
        // TRANSLATORS: %(duration)s - remaining time, e.g. "2 days"
        messages.pgettext('notifications', 'Account credit expires in %(duration)s'),
        { duration },
      ),
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
