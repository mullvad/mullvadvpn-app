import { isNumber } from '../../../../../../../../../shared/utils';
import { useAppUpgradeEvent } from '../../../../../../../../redux/hooks';
import { translations } from '../constants';

export const useGetMessageTimeLeft = () => {
  const { event } = useAppUpgradeEvent();

  const getMessageTimeLeft = () => {
    if (event?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { timeLeft } = event;
      const isTimeLeftNumeric = isNumber(timeLeft);

      if (isTimeLeftNumeric) {
        if (timeLeft > 90) {
          const minutes = Math.round(timeLeft / 60);

          return translations.getDownloadMinutesRemaining(minutes);
        }

        if (timeLeft > 3) {
          return translations.getDownloadSecondsRemaining(timeLeft);
        }

        return translations.downloadFewSecondsRemaining;
      }
    }

    return null;
  };

  return getMessageTimeLeft;
};
