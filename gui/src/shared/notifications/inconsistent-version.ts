import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface InconsistentVersionNotificationContext {
  consistent: boolean;
}

export class InconsistentVersionNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
  public constructor(private context: InconsistentVersionNotificationContext) {}

  public mayDisplay = () => !this.context.consistent;

  public getSystemNotification(): SystemNotification {
    return {
      message: messages.pgettext('notifications', 'App is out of sync. Please quit and restart.'),
      category: SystemNotificationCategory.inconsistentVersion,
      severity: SystemNotificationSeverityType.high,
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'APP IS OUT OF SYNC'),
      subtitle: messages.pgettext('in-app-notifications', 'Please quit and restart the app.'),
    };
  }
}
