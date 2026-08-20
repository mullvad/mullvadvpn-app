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
  platform: NodeJS.Platform;
}

export class InconsistentVersionNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider
{
  public constructor(private context: InconsistentVersionNotificationContext) {}

  public mayDisplay = () =>
    !this.context.consistent &&
    process.env.NODE_ENV !== undefined &&
    process.env.NODE_ENV !== 'development';

  public getSystemNotification(): SystemNotification {
    const message =
      this.context.platform === 'linux'
        ? messages.pgettext(
            'notifications',
            'App is out of sync. Please quit and restart the app or daemon.',
          )
        : messages.pgettext('notifications', 'App is out of sync. Please quit and restart.');

    return {
      message,
      category: SystemNotificationCategory.inconsistentVersion,
      severity: SystemNotificationSeverityType.high,
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    const subtitle =
      this.context.platform === 'linux'
        ? messages.pgettext('in-app-notifications', 'Please quit and restart the app or daemon.')
        : messages.pgettext('in-app-notifications', 'Please quit and restart the app.');

    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'APP IS OUT OF SYNC'),
      subtitle,
    };
  }
}
