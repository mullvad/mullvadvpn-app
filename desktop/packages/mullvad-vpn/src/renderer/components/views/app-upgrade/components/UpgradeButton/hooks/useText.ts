import { AppUpgradeError } from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';
import {
  useAppUpgradeError,
  useAppUpgradeEventType,
  useIsAppUpgradeDownloaded,
} from '../../../hooks';

export const useText = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const isAppUpgradeDownloaded = useIsAppUpgradeDownloaded();
  const appUpgradeError = useAppUpgradeError();

  if (isAppUpgradeDownloaded) {
    const appUpgradeEventAborted = appUpgradeEventType === 'APP_UPGRADE_EVENT_ABORTED';
    const hasErrorStartInstallerFailed = appUpgradeError === AppUpgradeError.startInstallerFailed;
    if (appUpgradeEventAborted || hasErrorStartInstallerFailed) {
      // TRANSLATORS: Button text to install an update
      return messages.pgettext('app-upgrade-view', 'Install update');
    }
  }

  if (appUpgradeEventType === 'APP_UPGRADE_EVENT_INSTALLER_READY') {
    // TRANSLATORS: Button text to install an update
    return messages.pgettext('app-upgrade-view', 'Install update');
  }

  if (
    appUpgradeError === AppUpgradeError.downloadFailed ||
    appUpgradeError === AppUpgradeError.generalError ||
    appUpgradeError === AppUpgradeError.verificationFailed
  ) {
    // TRANSLATORS: Button text to retry download of an update
    return messages.pgettext('app-upgrade-view', 'Retry download');
  }

  // TRANSLATORS: Button text to download and install an update
  return messages.pgettext('app-upgrade-view', 'Download and install');
};
