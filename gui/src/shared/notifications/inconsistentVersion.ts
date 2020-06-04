import { messages } from '../../shared/gettext';
import { NotificationIndicatorType } from './notification';

export const inconsistentVersion = {
  condition: (consistent: boolean) => !consistent,
  systemNotification: {
    message: messages.pgettext(
      'notifications',
      'Inconsistent internal version information, please restart the app',
    ),
    important: true,
    supressInDevelopment: true,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'INCONSISTENT VERSION'),
    body: messages.pgettext(
      'in-app-notifications',
      'Inconsistent internal version information, please restart the app',
    ),
  },
};
