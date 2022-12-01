import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

export class ReconnectingNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
  public constructor(private context: TunnelState) {}

  public mayDisplay() {
    return this.context.state === 'disconnecting' && this.context.details === 'reconnect';
  }

  public getSystemNotification(): SystemNotification | undefined {
    return {
      message: messages.pgettext('notifications', 'Reconnecting'),
      severity: SystemNotificationSeverityType.info,
      category: SystemNotificationCategory.tunnelState,
      throttle: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
    };
  }
}
