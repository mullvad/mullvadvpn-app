import { messages } from '../../shared/gettext';
import {
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

export class DaemonDisconnectedNotificationProvider implements SystemNotificationProvider {
  public mayDisplay = () => true;

  public getSystemNotification(): SystemNotification {
    return {
      message: messages.pgettext(
        'notifications',
        'Connection might be unsecured. App lost contact with system service, please troubleshoot.',
      ),
      severity: SystemNotificationSeverityType.high,
      category: SystemNotificationCategory.tunnelState,
    };
  }
}
