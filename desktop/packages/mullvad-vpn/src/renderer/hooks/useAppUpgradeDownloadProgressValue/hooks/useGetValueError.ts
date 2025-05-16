import { useCallback } from 'react';

import { useAppUpgradeError, useAppUpgradeLastProgress } from '../../../redux/hooks';
import { DOWNLOAD_COMPLETE_VALUE } from '../constants';
import { useGetValueDownloadProgress } from './useGetValueDownloadProgress';

export const useGetValueError = () => {
  const { error } = useAppUpgradeError();
  const getValueDownloadProgress = useGetValueDownloadProgress();
  const { lastProgress } = useAppUpgradeLastProgress();

  const getValueError = useCallback(() => {
    if (error === 'DOWNLOAD_FAILED' || error === 'GENERAL_ERROR') {
      return getValueDownloadProgress();
    }

    if (
      error === 'INSTALLER_FAILED' ||
      error === 'START_INSTALLER_FAILED' ||
      error === 'VERIFICATION_FAILED'
    ) {
      return DOWNLOAD_COMPLETE_VALUE;
    }

    return lastProgress;
  }, [error, getValueDownloadProgress, lastProgress]);

  return getValueError;
};
