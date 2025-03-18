import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../../../redux/hooks';

export const useGetTextError = () => {
  const { appUpgradeError } = useAppUpgradeError();

  const getTextError = () => {
    if (
      appUpgradeError === 'START_INSTALLER_FAILED' ||
      appUpgradeError === 'START_INSTALLER_AUTOMATIC_FAILED' ||
      appUpgradeError === 'VERIFICATION_FAILED'
    ) {
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
      return messages.pgettext('app-upgrade-view', 'Download complete!');
    }

    return null;
  };

  return getTextError;
};
