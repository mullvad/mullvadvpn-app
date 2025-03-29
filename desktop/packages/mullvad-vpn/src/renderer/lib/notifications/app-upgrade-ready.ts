import { sprintf } from 'sprintf-js';

import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../routes';

interface AppUpgradeReadyNotificationContext {
  shouldAppUpgradeInstallManually: boolean;
  suggestedUpgradeVersion?: string;
}

export class AppUpgradeReadyNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: AppUpgradeReadyNotificationContext) {}

  public mayDisplay = () => {
    return this.context.shouldAppUpgradeInstallManually;
  };

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'warning',
      title:
        // TRANSLATORS: Notification title when the app upgrade is ready to install.
        messages.pgettext('in-app-notifications', 'READY TO INSTALL UPDATE'),
      subtitle: [
        {
          content: sprintf(
            // TRANSLATORS: Notification subtitle when the app upgrade is ready to install.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: - %(suggestedUpgradeVersion)s: Upgrade version to be installed.
            messages.pgettext(
              'in-app-notifications',
              '%(suggestedUpgradeVersion)s is ready to be installed.',
            ),
            {
              suggestedUpgradeVersion: this.context.suggestedUpgradeVersion,
            },
          ),
        },
        {
          content:
            // TRANSLATORS: Notification subtitle when the app upgrade is ready to install.
            messages.pgettext('in-app-notifications', 'Click here to install update.'),
          action: {
            type: 'navigate-internal',
            link: {
              // TODO: Change route
              to: RoutePath.changelog,
              'aria-label':
                // TRANSLATORS: Accessibility label for link to app upgrade view.
                messages.pgettext('accessibility', 'Go to app upgrade'),
            },
          },
        },
      ],
    };
  }
}
