import { AppUpgradeError } from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../hooks';

export const useGetTextError = () => {
  const appUpgradeError = useAppUpgradeError();

  const getTextError = () => {
    if (
      appUpgradeError === AppUpgradeError.verificationFailed ||
      appUpgradeError === AppUpgradeError.startInstallerFailed
    ) {
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
      return messages.pgettext('app-upgrade-view', 'Download complete!');
    }

    return null;
  };

  return getTextError;
};
