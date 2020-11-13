export type NotificationAction = {
  type: 'open-url';
  url: string;
  text?: string;
  withAuth?: boolean;
};

export type InAppNotificationIndicatorType = 'success' | 'warning' | 'error';

interface NotificationProvider {
  mayDisplay(): boolean;
}

export interface SystemNotification {
  message: string;
  critical: boolean;
  presentOnce?: { value: boolean; name: string };
  suppressInDevelopment?: boolean;
  action?: NotificationAction;
}

export interface InAppNotification {
  indicator?: InAppNotificationIndicatorType;
  title: string;
  subtitle?: string;
  action?: NotificationAction;
}

export interface SystemNotificationProvider extends NotificationProvider {
  getSystemNotification(): SystemNotification | undefined;
}

export interface InAppNotificationProvider extends NotificationProvider {
  getInAppNotification(): InAppNotification | undefined;
}

export * from './account-expired';
export * from './close-to-account-expiry';
export * from './block-when-disconnected';
export * from './connected';
export * from './connecting';
export * from './disconnected';
export * from './error';
export * from './no-valid-key';
export * from './inconsistent-version';
export * from './reconnecting';
export * from './unsupported-version';
export * from './update-available';
