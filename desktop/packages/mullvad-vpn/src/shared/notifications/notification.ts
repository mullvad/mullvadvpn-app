import { LinkProps } from '../../renderer/lib/components';

export type NotificationAction = {
  type: 'open-url';
  url: string;
  text?: string;
  withAuth?: boolean;
};

export interface InAppNotificationTroubleshootInfo {
  details: string;
  steps: string[];
  buttons?: Array<InAppNotificationTroubleshootButton>;
}

export interface InAppNotificationTroubleshootButton {
  label: string;
  action: () => void;
}

export type InAppNotificationAction =
  | NotificationAction
  | {
      type: 'troubleshoot-dialog';
      troubleshoot: InAppNotificationTroubleshootInfo;
    }
  | {
      type: 'close';
      close: () => void;
    }
  | {
      type: 'navigate';
      link: Pick<LinkProps, 'to' | 'onClick' | 'aria-label'>;
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
  action?: InAppNotificationAction;
  title: string;
  subtitle?: string;
  subtitleAction?: InAppNotificationAction;
}

export interface SystemNotificationProvider extends NotificationProvider {
  getSystemNotification(): SystemNotification | undefined;
}

export interface InAppNotificationProvider extends NotificationProvider {
  getInAppNotification(): InAppNotification | undefined;
}
