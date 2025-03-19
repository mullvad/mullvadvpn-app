import { useAppUpgradeEvent } from '../../../../../hooks';
import { FALLBACK_VALUE } from '../constants';

export const useGetValueDownloadProgress = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const getValueDownloadProgress = () => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS') {
      const { progress } = appUpgradeEvent;

      return progress;
    }

    return FALLBACK_VALUE;
  };

  return getValueDownloadProgress;
};
