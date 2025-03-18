import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../hooks';

export const useGetTextTimeLeft = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const getTextTimeLeft = () => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { timeLeft } = appUpgradeEvent;

      // TODO: The cut off point for showing seconds or minutes is arbitrary
      if (timeLeft > 90) {
        // TODO: This rounding can be improved
        const minutes = Math.round(timeLeft / 60);

        return sprintf(
          // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
          messages.pgettext('app-upgrade-view', 'About %(minutes)s minutes remaining...'),
          {
            minutes,
          },
        );
      }

      return sprintf(
        // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
        messages.pgettext('app-upgrade-view', 'About %(seconds)s seconds remaining...'),
        {
          seconds: timeLeft,
        },
      );
    }

    return null;
  };

  return getTextTimeLeft;
};

export default useGetTextTimeLeft;
