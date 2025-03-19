import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';
import { useCanStartInstaller } from './useCanStartInstaller';

export const useHandleOnClick = () => {
  const { appUpgrade, appUpgradeInstall } = useAppContext();
  const canStartInstaller = useCanStartInstaller();

  const handleOnClick = useCallback(() => {
    if (canStartInstaller) {
      appUpgradeInstall();
    } else {
      appUpgrade();
    }
  }, [appUpgrade, appUpgradeInstall, canStartInstaller]);

  return handleOnClick;
};
