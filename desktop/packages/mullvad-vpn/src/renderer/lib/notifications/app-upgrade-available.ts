import { sprintf } from 'sprintf-js';

import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { getDownloadUrl } from '../../../shared/version';
import { RoutePath } from '../routes';

interface AppUpgradeAvailableNotificationContext {
  suggestedUpgradeVersion?: string;
  suggestedIsBeta?: boolean;
  updateDismissedForVersion?: string;
  platform: NodeJS.Platform;
  close: () => void;
}

export class AppUpgradeAvailableNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: AppUpgradeAvailableNotificationContext) {}

  public mayDisplay(): boolean {
    const { suggestedUpgradeVersion, suggestedIsBeta, updateDismissedForVersion } = this.context;
    if (!suggestedUpgradeVersion) {
      return false;
    }
    if (suggestedIsBeta && suggestedUpgradeVersion === updateDismissedForVersion) {
      return false;
    }
    return true;
  }

  public getInAppNotification(): InAppNotification {
    const { close, platform, suggestedIsBeta } = this.context;
    const isLinux = platform === 'linux';

    return {
      indicator: 'warning',
      title: suggestedIsBeta
        ? messages.pgettext('in-app-notifications', 'BETA UPDATE AVAILABLE')
        : messages.pgettext('in-app-notifications', 'UPDATE AVAILABLE'),
      subtitle: [
        {
          content: this.inAppMessage(),
        },
        {
          content:
            // TRANSLATORS: Link text to go to the app upgrade view
            messages.pgettext('in-app-notifications', 'Click here to update'),
          action: isLinux
            ? {
                type: 'navigate-external',
                link: {
                  to: getDownloadUrl(suggestedIsBeta ?? false),
                  'aria-label':
                    // TRANSLATORS: Accessbility label for link to go to download page.
                    messages.pgettext(
                      'accessibility',
                      'New version available, click here to go to download page, opens externally',
                    ),
                },
              }
            : {
                type: 'navigate-internal',
                link: {
                  to: RoutePath.changelog,
                  // TRANSLATORS: Accessbility label for link to go to upgrade view.
                  'aria-label': messages.pgettext(
                    'accessibility',
                    'New version available, click here to go to update view',
                  ),
                },
              },
        },
      ],
      action: suggestedIsBeta ? { type: 'close', close } : undefined,
    };
  }

  private inAppMessage(): string {
    const { suggestedIsBeta, suggestedUpgradeVersion } = this.context;
    if (suggestedIsBeta) {
      return sprintf(
        // TRANSLATORS: The in-app banner displayed to the user when the app beta update is
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(version)s - The version number of the new beta version.
        messages.pgettext('in-app-notifications', 'Try out the newest beta version (%(version)s).'),
        { version: suggestedUpgradeVersion },
      );
    } else {
      // TRANSLATORS: The in-app banner displayed to the user when the app update is available.
      return messages.pgettext(
        'in-app-notifications',
        'Install the latest app version to stay up to date.',
      );
    }
  }
}
