import { AppUpgradeError } from '../../../../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../../../../../hooks';

export const useGetTextError = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const getTextError = () => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_ERROR') {
      const { error } = appUpgradeEvent;

      if (
        error === AppUpgradeError.verificationFailed ||
        error === AppUpgradeError.startInstallerFailed
      ) {
        // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
        return messages.pgettext('download-update-view', 'Download complete!');
      }
    }

    return null;
  };

  return getTextError;
};
