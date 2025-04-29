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
  restartAppUpgradeInstaller: () => void;
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
        // TRANSLATORS: Notification subtitle when the download of the installer failed
        // TRANSLATORS: and the user can try downloading again.
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

      if (appUpgradeError === 'START_INSTALLER_FAILED') {
        return {
          indicator: 'error',
          title:
            // TRANSLATORS: Notification title when the installer failed.
            messages.pgettext('in-app-notifications', 'INSTALLER FAILED'),
          subtitle: [
            {
              content:
                // TRANSLATORS: Notification subtitle when the installer failed.
                messages.pgettext(
                  'in-app-notifications',
                  'The installer did not start successfully.',
                ),
            },
            {
              content:
                // TRANSLATORS: Notification subtitle when the installer failed.
                messages.pgettext('in-app-notifications', 'Click here to retry'),
              action: {
                type: 'run-function',
                button: {
                  onClick: () => this.context.restartAppUpgradeInstaller(),
                  'aria-label':
                    // TRANSLATORS: Accessibility label for the button to retry the installation.
                    messages.pgettext('in-app-notifications', 'Retry installation'),
                },
              },
            },
          ],
        };
      }

      if (appUpgradeError === 'INSTALLER_FAILED') {
        return {
          indicator: 'error',
          title:
            // TRANSLATORS: Notification title when the installer failed.
            messages.pgettext('in-app-notifications', 'INSTALLER FAILED'),
          subtitle: [
            {
              content:
                // TRANSLATORS: Notification subtitle when the installer failed.
                messages.pgettext(
                  'in-app-notifications',
                  'The installer did not complete successfully.',
                ),
            },
            {
              content:
                // TRANSLATORS: Notification subtitle when the installer failed.
                messages.pgettext('in-app-notifications', 'Click here to retry'),
              action: {
                type: 'run-function',
                button: {
                  onClick: () => this.context.restartAppUpgradeInstaller(),
                  'aria-label':
                    // TRANSLATORS: Accessibility label for the button to retry the installation.
                    messages.pgettext('in-app-notifications', 'Retry installation'),
                },
              },
            },
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
