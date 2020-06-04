import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationIndicatorType,
  NotificationProvider,
  SystemNotification,
} from './notification';

interface InconsistentVersionNotificationContext {
  consistent: boolean;
}

export class InconsistentVersionNotificationProvider
  extends NotificationProvider<InconsistentVersionNotificationContext>
  implements SystemNotification, InAppNotification {
  public get visible() {
    return !this.context.consistent;
  }

  public message = messages.pgettext(
    'notifications',
    'Inconsistent internal version information, please restart the app',
  );

  public critical = true;

  public presentOnce = true;

  public suppressInDevelopment = true;

  public indicator: InAppNotificationIndicatorType = 'error';

  public title = messages.pgettext('in-app-notifications', 'INCONSISTENT VERSION');

  public body = messages.pgettext(
    'in-app-notifications',
    'Inconsistent internal version information, please restart the app',
  );
}
