export type NotificationAction = {
  type: 'open-url';
  url: string;
  text?: string;
  withAuth?: boolean;
};

export interface InAppNotificationTroubleshootInfo {
  details: string;
  steps: string[];
}

export type InAppNotificationAction =
  | NotificationAction
  | {
      type: 'troubleshoot-dialog';
      troubleshoot: InAppNotificationTroubleshootInfo;
    };

export type InAppNotificationIndicatorType = 'success' | 'warning' | 'error';

export enum SystemNotificationSeverityType {
  info = 0,
  low,
  medium,
  high,
}

export enum SystemNotificationCategory {
  tunnelState,
  expiry,
  newVersion,
  inconsistentVersion,
}

interface NotificationProvider {
  mayDisplay(): boolean;
}

export interface SystemNotification {
  message: string;
  severity: SystemNotificationSeverityType;
  category: SystemNotificationCategory;
  throttle?: boolean;
  presentOnce?: { value: boolean; name: string };
  suppressInDevelopment?: boolean;
  action?: NotificationAction;
}

export interface InAppNotification {
  indicator?: InAppNotificationIndicatorType;
  title: string;
  subtitle?: string;
  action?: InAppNotificationAction;
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
export * from './inconsistent-version';
export * from './reconnecting';
export * from './unsupported-version';
export * from './update-available';
