import { messages } from '../../../../../../../../../../../../../shared/gettext';
import { useAppUpgradeError } from '../../../../../../../../../../../../redux/hooks';

export const useMessage = () => {
  const { error } = useAppUpgradeError();

  if (error === 'DOWNLOAD_FAILED') {
    // TRANSLATORS: Button text to try download again
    return messages.pgettext('app-upgrade-view', 'Retry download');
  }

  // TRANSLATORS: Button text to try again
  return messages.pgettext('app-upgrade-view', 'Retry');
};
