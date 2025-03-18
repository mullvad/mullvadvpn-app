import { messages } from '../../../../../../../shared/gettext';
import {
  useAppUpgradeError,
  useAppUpgradeEventType,
  useIsAppUpgradeInstallerReady,
} from '../../../hooks';

export const useText = () => {
  const appUpgradeError = useAppUpgradeError();
  const appUpgradeEventType = useAppUpgradeEventType();
  const isAppUpgradeInstallerReady = useIsAppUpgradeInstallerReady();

  if (isAppUpgradeInstallerReady) {
    const appUpgradeEventAborted = appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED';
    const hasErrorStartInstallerFailed = appUpgradeError === 'START_INSTALLER_FAILED';

    if (appUpgradeEventAborted || hasErrorStartInstallerFailed) {
      // TRANSLATORS: Button text to install an update
      return messages.pgettext('app-upgrade-view', 'Install update');
    }
  }

  if (
    appUpgradeError === 'DOWNLOAD_FAILED' ||
    appUpgradeError === 'GENERAL_ERROR' ||
    appUpgradeError === 'VERIFICATION_FAILED'
  ) {
    // TRANSLATORS: Button text to retry download of an update
    return messages.pgettext('app-upgrade-view', 'Retry download');
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
      return messages.pgettext('app-upgrade-view', 'Install update');
    default:
      // TRANSLATORS: Button text to download and install an update
      return messages.pgettext('app-upgrade-view', 'Download and install');
  }
};
