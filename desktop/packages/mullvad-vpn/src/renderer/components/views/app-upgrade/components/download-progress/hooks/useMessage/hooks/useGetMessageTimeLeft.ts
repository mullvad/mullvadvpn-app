import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../shared/gettext';
import { isNumber } from '../../../../../../../../../shared/utils';
import { useAppUpgradeEvent } from '../../../../../../../../redux/hooks';

export const useGetMessageTimeLeft = () => {
  const { appUpgradeEvent } = useAppUpgradeEvent();

  const getMessageTimeLeft = () => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { timeLeft } = appUpgradeEvent;
      const isTimeLeftNumeric = isNumber(timeLeft);

      if (isTimeLeftNumeric) {
        if (timeLeft > 90) {
          const minutes = Math.round(timeLeft / 60);

          return sprintf(
            // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(minutes)s - Will be replaced with remaining minutes until download is complete
            messages.pgettext('app-upgrade-view', 'About %(minutes)s minutes remaining...'),
            {
              minutes,
            },
          );
        }

        if (timeLeft > 5) {
          return sprintf(
            // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(second)s - Will be replaced with remaining seconds until download is complete
            messages.pgettext('app-upgrade-view', 'About %(seconds)s seconds remaining...'),
            {
              seconds: timeLeft,
            },
          );
        }

        return sprintf(
          // TRANSLATORS: Status text displayed below a progress bar when the update is being downloaded
          messages.pgettext('app-upgrade-view', 'A few seconds remaining...'),
          {
            seconds: timeLeft,
          },
        );
      }
    }

    return null;
  };

  return getMessageTimeLeft;
};
