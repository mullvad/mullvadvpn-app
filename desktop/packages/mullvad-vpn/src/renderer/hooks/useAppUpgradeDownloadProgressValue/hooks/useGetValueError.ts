import { useCallback } from 'react';

import { useAppUpgradeError } from '../../../redux/hooks';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from '../constants';
import { useGetValueDownloadProgress } from './useGetValueDownloadProgress';

export const useGetValueError = () => {
  const { error } = useAppUpgradeError();
  const getValueDownloadProgress = useGetValueDownloadProgress();

  const getValueError = useCallback(() => {
    if (error === 'DOWNLOAD_FAILED' || error === 'GENERAL_ERROR') {
      return getValueDownloadProgress();
    }

    if (
      error === 'START_INSTALLER_AUTOMATIC_FAILED' ||
      error === 'START_INSTALLER_FAILED' ||
      error === 'VERIFICATION_FAILED'
    ) {
      return DOWNLOAD_COMPLETE_VALUE;
    }

    return FALLBACK_VALUE;
  }, [error, getValueDownloadProgress]);

  return getValueError;
};
