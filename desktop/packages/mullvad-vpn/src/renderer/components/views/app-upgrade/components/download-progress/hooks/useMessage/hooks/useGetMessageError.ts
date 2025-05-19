import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../../../redux/hooks';

export const useGetMessageError = () => {
  const { error } = useAppUpgradeError();

  const getMessageError = () => {
    if (
      error === 'INSTALLER_FAILED' ||
      error === 'START_INSTALLER_FAILED' ||
      error === 'VERIFICATION_FAILED'
    ) {
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
      return messages.pgettext('app-upgrade-view', 'Download complete!');
    }

    if (error === 'DOWNLOAD_FAILED') {
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update fails
      return messages.pgettext('app-upgrade-view', 'Download failed');
    }

    return null;
  };

  return getMessageError;
};
