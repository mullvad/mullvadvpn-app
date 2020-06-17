import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationProvider,
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
      message: messages.pgettext(
        'notifications',
        'Inconsistent internal version information, please restart the app',
      ),
      critical: true,
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'INCONSISTENT VERSION'),
      subtitle: messages.pgettext(
        'in-app-notifications',
        'Inconsistent internal version information, please restart the app',
      ),
    };
  }
}
