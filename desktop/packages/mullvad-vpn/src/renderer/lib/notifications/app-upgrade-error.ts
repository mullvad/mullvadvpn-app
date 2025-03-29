import { AppUpgradeError } from '../../../shared/app-upgrade';
import { messages } from '../../../shared/gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  InAppNotificationSubtitle,
} from '../../../shared/notifications';

interface AppUpgradeErrorNotificationContext {
  hasAppUpgradeError: boolean;
  appUpgradeError?: AppUpgradeError;
  restartAppUpgrade: () => void;
}

export class AppUpgradeErrorNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: AppUpgradeErrorNotificationContext) {}

  public mayDisplay = () => {
    return this.context.hasAppUpgradeError;
  };

  public getInAppNotification(): InAppNotification {
    const { appUpgradeError } = this.context;
    const retrySubtitle: InAppNotificationSubtitle = {
      content:
        // TRANSLATORS: Notification subtitle when the installer verification failed.
        messages.pgettext('in-app-notifications', 'Click here to retry download'),
      action: {
        type: 'run-function',
        button: {
          onClick: () => this.context.restartAppUpgrade(),
          'aria-label':
            // TRANSLATORS: Accessibility label for the button to retry download of the installer.
            messages.pgettext('in-app-notifications', 'Retry download of the installer'),
        },
      },
    };

    if (appUpgradeError) {
      if (appUpgradeError === 'VERIFICATION_FAILED') {
        return {
          indicator: 'error',
          title:
            // TRANSLATORS: Notification title when the installer verification failed.
            messages.pgettext('in-app-notifications', 'VERIFICATION FAILED'),
          subtitle: [
            {
              content:
                // TRANSLATORS: Notification subtitle when the installer verification failed.
                messages.pgettext('in-app-notifications', 'Installer could not be verified.'),
            },
            retrySubtitle,
          ],
        };
      }
      if (appUpgradeError === 'DOWNLOAD_FAILED') {
        return {
          indicator: 'error',
          title:
            // TRANSLATORS: Notification title when the installer download failed.
            messages.pgettext('in-app-notifications', 'DOWNLOAD FAILED'),
          subtitle: [
            {
              content:
                // TRANSLATORS: Notification subtitle when the installer download failed.
                messages.pgettext('in-app-notifications', 'Could not download installer.'),
            },
            retrySubtitle,
          ],
        };
      }
    }

    return {
      indicator: 'error',
      title:
        // TRANSLATORS: Generic notification title when the app upgrade failed.
        messages.pgettext('in-app-notifications', 'UPDATE FAILED'),
      subtitle: [
        {
          content:
            // TRANSLATORS: Generic notification subtitle when the app upgrade failed.
            messages.pgettext('in-app-notifications', 'Could not upgrade the app.'),
        },
        retrySubtitle,
      ],
    };
  }
}
