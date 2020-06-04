export type NotificationAction = { type: 'open-url'; url: string; withAuth?: boolean };

export type InAppNotificationIndicatorType = 'success' | 'warning' | 'error';

interface Notification {
  visible: boolean;
  action?: NotificationAction;
}

export interface SystemNotification extends Notification {
  message: string;
  critical: boolean;
  presentOnce?: boolean;
  suppressInDevelopment?: boolean;
}

export interface InAppNotification extends Notification {
  indicator: InAppNotificationIndicatorType;
  title: string;
  body?: string;
}

export class NotificationProvider<TContext> {
  constructor(protected context: TContext) {}
}

export * from './accountExpiry';
export * from './blockWhenDisconnected';
export * from './connected';
export * from './connecting';
export * from './disconnected';
export * from './error';
export * from './inconsistentVersion';
export * from './reconnecting';
export * from './unsupportedVersion';
export * from './updateAvailable';
