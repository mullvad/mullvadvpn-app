import { app, NativeImage, nativeImage, Notification as ElectronNotification } from 'electron';
import os from 'os';
import path from 'path';

import { TunnelState } from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import {
  ConnectedNotificationProvider,
  ConnectingNotificationProvider,
  DisconnectedNotificationProvider,
  ErrorNotificationProvider,
  NotificationAction,
  ReconnectingNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from '../shared/notifications/notification';
import { Scheduler } from '../shared/scheduler';

const THROTTLE_DELAY = 500;

export interface Notification {
  specification: SystemNotification;
  notification: ElectronNotification;
}

export interface NotificationSender {
  notify(notification: SystemNotification): void;
  closeNotificationsInCategory(category: SystemNotificationCategory): void;
}

export interface NotificationControllerDelegate {
  openApp(): void;
  openLink(url: string, withAuth?: boolean): Promise<void>;
}

export default class NotificationController {
  private reconnecting = false;
  private presentedNotifications: { [key: string]: boolean } = {};
  private activeNotifications: Set<Notification> = new Set();
  private throttledNotifications: Map<SystemNotification, Scheduler> = new Map();
  private notificationTitle = process.platform === 'linux' ? app.name : '';
  private notificationIcon?: NativeImage;

  constructor(private notificationControllerDelegate: NotificationControllerDelegate) {
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

  public dispose() {
    this.throttledNotifications.forEach((scheduler) => scheduler.cancel());

    this.activeNotifications.forEach((notification) => notification.notification.close());
    this.activeNotifications.clear();
  }

  public notifyTunnelState(
    tunnelState: TunnelState,
    blockWhenDisconnected: boolean,
    hasExcludedApps: boolean,
    isWindowVisible: boolean,
    areSystemNotificationsEnabled: boolean,
  ) {
    const notificationProviders: SystemNotificationProvider[] = [
      new ConnectingNotificationProvider({ tunnelState, reconnecting: this.reconnecting }),
      new ConnectedNotificationProvider(tunnelState),
      new ReconnectingNotificationProvider(tunnelState),
      new DisconnectedNotificationProvider({ tunnelState, blockWhenDisconnected }),
      new ErrorNotificationProvider({ tunnelState, hasExcludedApps }),
    ];

    const notificationProvider = notificationProviders.find((notification) =>
      notification.mayDisplay(),
    );

    if (notificationProvider) {
      const notification = notificationProvider.getSystemNotification();

      if (notification) {
        this.notify(notification, isWindowVisible, areSystemNotificationsEnabled);
      } else {
        log.error(
          `Notification providers mayDisplay() returned true but getSystemNotification() returned undefined for ${notificationProvider.constructor.name}`,
        );
      }
    } else {
      this.closeNotificationsInCategory(SystemNotificationCategory.tunnelState);
    }

    this.reconnecting =
      tunnelState.state === 'disconnecting' && tunnelState.details === 'reconnect';
  }

  public closeActiveNotifications() {
    this.activeNotifications.forEach((notification) => notification.notification.close());
  }

  public closeNotificationsInCategory(category: SystemNotificationCategory) {
    this.activeNotifications.forEach((notification) => {
      if (notification.specification.category === category) {
        notification.notification.close();
      }
    });
  }

  public notify(
    systemNotification: SystemNotification,
    windowVisible: boolean,
    infoNotificationsEnabled: boolean,
  ) {
    if (!this.evaluateNotification(systemNotification, windowVisible, infoNotificationsEnabled)) {
      return;
    }

    // Cancel throttled notifications within the same category
    if (systemNotification.category !== undefined) {
      this.throttledNotifications.forEach((scheduler, specification) => {
        if (specification.category === systemNotification.category) {
          scheduler.cancel();
          this.throttledNotifications.delete(specification);
        }
      });
    }

    if (systemNotification.throttle) {
      const scheduler = new Scheduler();
      scheduler.schedule(() => {
        this.throttledNotifications.delete(systemNotification);
        this.notifyImpl(systemNotification);
      }, THROTTLE_DELAY);

      this.throttledNotifications.set(systemNotification, scheduler);
    } else {
      this.notifyImpl(systemNotification);
    }
  }

  private notifyImpl(systemNotification: SystemNotification): Notification {
    // Remove notifications in the same category if specified
    if (systemNotification.category !== undefined) {
      this.closeNotificationsInCategory(systemNotification.category);
    }

    const notification = this.createNotification(systemNotification);
    this.addActiveNotification(notification);
    notification.notification.show();

    // Close notification of low severity automatically
    if (systemNotification.severity === SystemNotificationSeverityType.info) {
      setTimeout(() => notification.notification.close(), 4000);
    }

    return notification;
  }

  private createNotification(systemNotification: SystemNotification): Notification {
    const notification = new ElectronNotification({
      title: this.notificationTitle,
      body: systemNotification.message,
      silent: true,
      icon: this.notificationIcon,
      timeoutType:
        systemNotification.severity == SystemNotificationSeverityType.high ? 'never' : 'default',
    });

    // Action buttons are only available on macOS.
    if (process.platform === 'darwin') {
      if (systemNotification.action) {
        notification.actions = [{ type: 'button', text: systemNotification.action.text }];
        notification.on('action', () => this.performAction(systemNotification.action));
      }
      notification.on('click', () => this.notificationControllerDelegate.openApp());
    } else if (
      !(
        process.platform === 'win32' &&
        systemNotification.severity === SystemNotificationSeverityType.high
      )
    ) {
      if (systemNotification.action) {
        notification.on('click', () => this.performAction(systemNotification.action));
      } else {
        notification.on('click', () => this.notificationControllerDelegate.openApp());
      }
    }

    return { specification: systemNotification, notification };
  }

  private performAction(action?: NotificationAction) {
    if (action && action.type === 'open-url') {
      void this.notificationControllerDelegate.openLink(action.url, action.withAuth);
    }
  }

  private addActiveNotification(notification: Notification) {
    notification.notification.on('close', () => this.activeNotifications.delete(notification));
    this.activeNotifications.add(notification);
  }

  private evaluateNotification(
    notification: SystemNotification,
    isWindowVisible: boolean,
    areSystemNotificationsEnabled: boolean,
  ) {
    const suppressDueToDevelopment =
      notification.suppressInDevelopment && process.env.NODE_ENV === 'development';
    const suppressDueToVisibleWindow = isWindowVisible;
    const suppressDueToPreference =
      !areSystemNotificationsEnabled && notification.severity > SystemNotificationSeverityType.info;

    return (
      !suppressDueToDevelopment &&
      !suppressDueToVisibleWindow &&
      !suppressDueToPreference &&
      !this.suppressDueToAlreadyPresented(notification)
    );
  }

  private suppressDueToAlreadyPresented(notification: SystemNotification) {
    const presented = this.presentedNotifications;
    if (notification.presentOnce?.value) {
      if (presented[notification.presentOnce.name]) {
        return true;
      } else {
        presented[notification.presentOnce.name] = true;
        return false;
      }
    } else {
      return false;
    }
  }
}
