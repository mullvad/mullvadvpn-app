import { nativeImage, NativeImage, Notification, shell } from 'electron';
import os from 'os';
import path from 'path';
import { sprintf } from 'sprintf-js';
import config from '../config.json';
import { TunnelState } from '../shared/daemon-rpc-types';
import { messages } from '../shared/gettext';

export default class NotificationController {
  private lastTunnelStateAnnouncement?: { body: string; notification: Notification };
  private reconnecting = false;
  private presentedNotifications: { [key: string]: boolean } = {};
  private pendingNotifications: Notification[] = [];
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
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Connecting'), msg);
          } else {
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Connecting'), '');
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
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Secured'), msg);
          } else {
            this.showTunnelStateNotification(messages.pgettext('notifications', 'Secured'), '');
          }
        }
        break;
      case 'disconnected':
        this.showTunnelStateNotification(messages.pgettext('notifications', 'Unsecured'), '');
        break;
      case 'blocked':
        {
          switch (tunnelState.details.reason) {
            case 'is_offline':
              this.showTunnelStateNotification(
                // TRANSLATORS: The notification title showed when the computer is offline.
                messages.pgettext('notifications', 'Computer appears to be offline'),
                // TRANSLATORS: The notification message body showed when the computer is offline.
                messages.pgettext(
                  'notifications',
                  'Connections are blocked due to connectivity issues',
                ),
              );
              break;
            default:
              this.showTunnelStateNotification(
                // TRANSLATORS: The notification title showed when the connection is lost due to an error or block.
                messages.pgettext('notifications', 'Critical failure'),
                // TRANSLATORS: The notification message body showed when the connection is lost due to an error or block.
                messages.pgettext('notifications', 'Blocked all connections'),
              );
              break;
          }
        }
        break;
      case 'disconnecting':
        switch (tunnelState.details) {
          case 'nothing':
          case 'block':
            // no-op
            break;
          case 'reconnect':
            this.showTunnelStateNotification(
              messages.pgettext('notifications', 'Reconnecting'),
              '',
            );
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
        // TRANSLATORS: The system notification title displayed when the version information is inconsistent.
        title: messages.pgettext('notifications', 'Inconsistent internal version information'),
        // TRANSLATORS: The system notification message body displayed when the version information is inconsistent.
        body: messages.pgettext('notifications', 'Please restart the app'),
        silent: true,
        icon: this.notificationIcon,
      });
      this.scheduleNotification(notification);
    });
  }

  public notifyUnsupportedVersion(upgradeVersion: string) {
    this.presentNotificationOnce('unsupported-version', () => {
      const notification = new Notification({
        // TRANSLATORS: The system notification title displayed to the user when the running app becomes unsupported.
        title: messages.pgettext('notifications', 'Unsupported app version'),
        body: sprintf(
          // TRANSLATORS: The system notification message body displayed to the user when the running app becomes unsupported.
          // TRANSLATORS: Available placeholder:
          // TRANSLATORS: %(version) - the newest available version of the app
          messages.pgettext(
            'notifications',
            'Please upgrade to %(version)s now to ensure your security',
          ),
          {
            version: upgradeVersion,
          },
        ),
        silent: true,
        icon: this.notificationIcon,
      });

      notification.on('click', () => {
        shell.openExternal(config.links.download);
      });

      this.scheduleNotification(notification);
    });
  }

  public notifyKeyGenerationFailed() {
    const notification = new Notification({
      title: '',
      body: messages.pgettext('notifications', 'Wireguard key generation failed'),
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

  private showTunnelStateNotification(title: string, message: string) {
    const lastAnnouncement = this.lastTunnelStateAnnouncement;
    const sameAsLastNotification = lastAnnouncement && lastAnnouncement.body === message;

    if (sameAsLastNotification) {
      return;
    }

    const newNotification = new Notification({
      title,
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
