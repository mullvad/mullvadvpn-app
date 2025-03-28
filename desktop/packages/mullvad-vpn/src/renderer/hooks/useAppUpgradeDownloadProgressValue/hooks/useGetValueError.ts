import { useCallback } from 'react';

import { useAppUpgradeError } from '../../../redux/hooks';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from '../constants';
import { useGetValueDownloadProgress } from './useGetValueDownloadProgress';

export const useGetValueError = () => {
  const { appUpgradeError } = useAppUpgradeError();
  const getValueDownloadProgress = useGetValueDownloadProgress();

  const getValueError = useCallback(() => {
    if (appUpgradeError === 'DOWNLOAD_FAILED' || appUpgradeError === 'GENERAL_ERROR') {
      return getValueDownloadProgress();
    }

    if (appUpgradeError === 'START_INSTALLER_FAILED' || appUpgradeError === 'VERIFICATION_FAILED') {
      return DOWNLOAD_COMPLETE_VALUE;
    }

    return FALLBACK_VALUE;
  }, [appUpgradeError, getValueDownloadProgress]);

  return getValueError;
};
