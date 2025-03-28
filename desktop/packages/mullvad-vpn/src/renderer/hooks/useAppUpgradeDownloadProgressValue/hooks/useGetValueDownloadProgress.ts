import { useCallback } from 'react';

import { useAppUpgradeEvent } from '../../../redux/hooks';
import { FALLBACK_VALUE } from '../constants';

export const useGetValueDownloadProgress = () => {
  const { appUpgradeEvent } = useAppUpgradeEvent();

  const getValueDownloadProgress = useCallback(() => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { progress } = appUpgradeEvent;

      return progress;
    }

    return FALLBACK_VALUE;
  }, [appUpgradeEvent]);

  return getValueDownloadProgress;
};
