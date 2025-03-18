import { messages } from '../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../redux/hooks';

export const useText = () => {
  const { appUpgradeError } = useAppUpgradeError();

  if (
    appUpgradeError === 'DOWNLOAD_FAILED' ||
    appUpgradeError === 'GENERAL_ERROR' ||
    appUpgradeError === 'VERIFICATION_FAILED'
  ) {
    // TRANSLATORS: Button text to retry download of an update
    return messages.pgettext('app-upgrade-view', 'Retry download');
  }

  // TRANSLATORS: Button text to download and install an update
  return messages.pgettext('app-upgrade-view', 'Download and install');
};
