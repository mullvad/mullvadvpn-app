import { AppUpgradeError } from '../../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../../../hooks';

export const useText = () => {
  const event = useAppUpgradeEvent();

  // TODO: Handle more cases when we should present 'Install Update' and
  // do it better/more concise than here.
  if (event?.type) {
    if (event.type === 'APP_UPGRADE_EVENT_ERROR') {
      if (event.error === AppUpgradeError.startInstallerFailed) {
        return messages.pgettext('download-update-view', 'Install update');
      }
    }

    if (event.type === 'APP_UPGRADE_EVENT_ABORTED') {
      const upgradeDownloaded = false;
      if (upgradeDownloaded) {
        // TRANSLATORS: Button text to install an update
        return messages.pgettext('download-update-view', 'Install update');
      }
    }
  }

  switch (event?.type) {
    case 'APP_UPGRADE_EVENT_ABORTED':
      // TRANSLATORS: Button text to install an update
      return messages.pgettext('download-update-view', 'Install update');
    case 'APP_UPGRADE_EVENT_STARTING_INSTALLER':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
      // TRANSLATORS: Button text to cancel the download of an update
      return messages.pgettext('download-update-view', 'Cancel');
    default:
      // TRANSLATORS: Button text to download and install an update
      return messages.pgettext('download-update-view', 'Download and install');
  }
};
