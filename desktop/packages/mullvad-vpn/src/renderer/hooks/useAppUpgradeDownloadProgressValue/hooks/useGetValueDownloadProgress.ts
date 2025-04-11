import { useCallback } from 'react';

import { useAppUpgradeEvent } from '../../../redux/hooks';
import { FALLBACK_VALUE } from '../constants';

export const useGetValueDownloadProgress = () => {
  const { event } = useAppUpgradeEvent();

  const getValueDownloadProgress = useCallback(() => {
    if (event?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { progress } = event;

      return progress;
    }

    return FALLBACK_VALUE;
  }, [event]);

  return getValueDownloadProgress;
};
