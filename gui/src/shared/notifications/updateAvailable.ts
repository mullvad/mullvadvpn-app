import { messages } from '../../shared/gettext';
import { NotificationIndicatorType } from './notification';

export const updateAvailable = {
  condition: (nextUpgrade: string | null, current: string) =>
    nextUpgrade && nextUpgrade !== current,
  inAppNotification: {
    indicator: 'warning' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'UPDATE AVAILABLE'),
    // TRANSLATORS: The in-app banner displayed to the user when the app update is available.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(version)s - the newest available version of the app
    body: messages.pgettext(
      'in-app-notifications',
      'Install Mullvad VPN (%(version)s) to stay up to date',
    ),
  },
};
