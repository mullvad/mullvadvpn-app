import { sprintf } from 'sprintf-js';

import { AppUpgradeEvent } from '../../../shared/app-upgrade';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';

interface AppUpgradeReadyNotificationContext {
  appUpgradeEventType?: AppUpgradeEvent['type'];
  suggestedUpgradeVersion?: string;
}

export class AppUpgradeReadyNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: AppUpgradeReadyNotificationContext) {}

  public mayDisplay = () => {
    return (
      this.context.appUpgradeEventType === 'APP_UPGRADE_STATUS_EXITED_INSTALLER' ||
      this.context.appUpgradeEventType === 'APP_UPGRADE_STATUS_MANUAL_START_INSTALLER'
    );
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
              to: RoutePath.appUpgrade,
              'aria-label':
                // TRANSLATORS: Accessibility label for link to app upgrade view.
                messages.pgettext('accessibility', 'Go to app update page'),
            },
          },
        },
      ],
    };
  }
}
