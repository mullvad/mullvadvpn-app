import { useCallback } from 'react';

import { useAppUpgradeEvent, useAppUpgradeLastProgress } from '../../../redux/hooks';

export const useGetValueDownloadProgress = () => {
  const { event } = useAppUpgradeEvent();
  const { lastProgress } = useAppUpgradeLastProgress();

  const getValueDownloadProgress = useCallback(() => {
    if (event?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { progress } = event;

      return progress;
    }

    return lastProgress;
  }, [event, lastProgress]);

  return getValueDownloadProgress;
};
