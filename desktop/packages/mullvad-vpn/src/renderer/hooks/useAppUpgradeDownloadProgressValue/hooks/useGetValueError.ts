import { useCallback } from 'react';

import { useAppUpgradeError } from '../../../redux/hooks';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from '../constants';

export const useGetValueError = () => {
  const { appUpgradeError } = useAppUpgradeError();

  const getValueError = useCallback(() => {
    if (appUpgradeError === 'START_INSTALLER_FAILED' || appUpgradeError === 'VERIFICATION_FAILED') {
      return DOWNLOAD_COMPLETE_VALUE;
    }

    return FALLBACK_VALUE;
  }, [appUpgradeError]);

  return getValueError;
};
