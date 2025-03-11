import { ExternalLinkProps } from '../../renderer/components/ExternalLink';
import { InternalLinkProps } from '../../renderer/components/InternalLink';
import { Url } from '../constants';

export type NotificationAction = {
  type: 'open-url';
  url: Url;
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
  variant?: 'primary' | 'success' | 'destructive';
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
      type: 'navigate-internal';
      link: Pick<InternalLinkProps, 'to' | 'onClick' | 'aria-label'>;
    }
  | {
      type: 'navigate-external';
      link: Pick<ExternalLinkProps, 'to' | 'onClick' | 'aria-label'>;
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
  subtitle?: string | InAppNotificationSubtitle[];
}

export interface InAppNotificationSubtitle {
  content: string;
  action?: InAppNotificationAction;
}

export interface SystemNotificationProvider extends NotificationProvider {
  getSystemNotification(): SystemNotification | undefined;
}

export interface InAppNotificationProvider extends NotificationProvider {
  getInAppNotification(): InAppNotification | undefined;
}
