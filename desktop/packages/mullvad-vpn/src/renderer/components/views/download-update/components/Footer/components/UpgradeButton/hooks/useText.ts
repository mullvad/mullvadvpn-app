import { AppUpgradeError } from '../../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../../shared/gettext';
import {
  useAppUpgradeEventType,
  useGetHasAppUpgradeError,
  useIsAppUpgradeDownloaded,
} from '../../../../../hooks';

export const useText = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const isAppUpgradeDownloaded = useIsAppUpgradeDownloaded();
  const getHasAppUpgradeError = useGetHasAppUpgradeError();

  if (isAppUpgradeDownloaded) {
    const appUpgradeEventAborted = appUpgradeEventType === 'APP_UPGRADE_EVENT_ABORTED';
    const hasErrorStartInstallerFailed = getHasAppUpgradeError(
      AppUpgradeError.startInstallerFailed,
    );
    if (appUpgradeEventAborted || hasErrorStartInstallerFailed) {
      // TRANSLATORS: Button text to install an update
      return messages.pgettext('download-update-view', 'Install update');
    }
  }

  if (appUpgradeEventType === 'APP_UPGRADE_EVENT_ERROR') {
    // TRANSLATORS: Button text to retry download of an update
    return messages.pgettext('download-update-view', 'Retry download');
  }

  // TRANSLATORS: Button text to download and install an update
  return messages.pgettext('download-update-view', 'Download and install');
};
