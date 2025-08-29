import { sprintf } from 'sprintf-js';

import { AppUpgradeEvent, AppUpgradeStep } from '../../../shared/app-upgrade';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';

interface AppUpgradeProgressNotificationContext {
  appUpgradeStep: AppUpgradeStep;
  appUpgradeDownloadProgressValue: number;
  appUpgradeEventType?: AppUpgradeEvent['type'];
}

export class AppUpgradeProgressNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: AppUpgradeProgressNotificationContext) {}

  public mayDisplay = () => {
    return (
      this.context.appUpgradeStep === 'download' ||
      this.context.appUpgradeStep === 'launch' ||
      this.context.appUpgradeStep === 'verify'
    );
  };

  public getInAppNotification(): InAppNotification {
    const { appUpgradeDownloadProgressValue, appUpgradeEventType } = this.context;
    if (appUpgradeEventType === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      return {
        indicator: 'warning',
        title: sprintf(
          // TRANSLATORS: Notification title when the app upgrade is in progress.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: - %(appUpgradeDownloadProgressValue)s: The download progress value.
          messages.pgettext(
            'in-app-notifications',
            'DOWNLOADING UPDATE... %(appUpgradeDownloadProgressValue)s%%',
          ),
          {
            appUpgradeDownloadProgressValue,
          },
        ),
      };
    }

    if (appUpgradeEventType === 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER') {
      return {
        indicator: 'warning',
        title:
          // TRANSLATORS: Notification title when app upgrade is verifying the installer.
          messages.pgettext('in-app-notifications', 'DOWNLOAD COMPLETE! VERIFYING...'),
      };
    }

    if (
      appUpgradeEventType === 'APP_UPGRADE_STATUS_MANUAL_START_INSTALLER' ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER' ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_EXITED_INSTALLER'
    ) {
      return {
        indicator: 'warning',
        title:
          // TRANSLATORS: Notification title when app upgrade is ready for the user to launch the installer.
          messages.pgettext('in-app-notifications', 'VERIFICATION COMPLETE! INSTALLER READY!'),
      };
    }

    if (
      appUpgradeEventType === 'APP_UPGRADE_STATUS_AUTOMATIC_STARTING_INSTALLER' ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_MANUAL_STARTING_INSTALLER' ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTED_INSTALLER'
    ) {
      return {
        indicator: 'warning',
        title:
          // TRANSLATORS: Notification title when app upgrade is launching the installer.
          messages.pgettext(
            'in-app-notifications',
            'VERIFICATION COMPLETE! LAUNCHING INSTALLER...',
          ),
      };
    }

    return {
      indicator: 'warning',
      title:
        // TRANSLATORS: Generic notification title when app upgrade is downloading the installer.
        messages.pgettext('in-app-notifications', 'DOWNLOADING UPDATE...'),
    };
  }
}
