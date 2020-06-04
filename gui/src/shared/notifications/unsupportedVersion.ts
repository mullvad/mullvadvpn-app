import { messages } from '../../shared/gettext';
import { NotificationIndicatorType } from './notification';

export const unsupportedVersion = {
  condition: (supported: boolean, consistent: boolean, nextUpgrade: string | null) =>
    consistent && !supported && nextUpgrade !== null,
  systemNotification: {
    // TRANSLATORS: The system notification displayed to the user when the running app becomes unsupported.
    // TRANSLATORS: Available placeholder:
    // TRANSLATORS: %(version) - the newest available version of the app
    message: messages.pgettext(
      'notifications',
      'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
    ),
    important: true,
    supressInDevelopment: true,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION'),
    // TRANSLATORS: The in-app banner displayed to the user when the running app becomes unsupported.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(version)s - the newest available version of the app
    body: messages.pgettext(
      'in-app-notifications',
      'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
    ),
  },
};
