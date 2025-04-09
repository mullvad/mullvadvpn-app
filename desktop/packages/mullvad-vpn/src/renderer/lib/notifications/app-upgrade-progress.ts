import { sprintf } from 'sprintf-js';

import { AppUpgradeEvent } from '../../../shared/app-upgrade';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';

interface AppUpgradeProgressNotificationContext {
  isAppUpgradeInProgress: boolean;
  appUpgradeDownloadProgressValue: number;
  appUpgradeEventType?: AppUpgradeEvent['type'];
}

export class AppUpgradeProgressNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: AppUpgradeProgressNotificationContext) {}

  public mayDisplay = () => {
    return this.context.isAppUpgradeInProgress;
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
      appUpgradeEventType === 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER' ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTING_INSTALLER' ||
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
