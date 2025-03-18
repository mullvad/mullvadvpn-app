import { useCallback } from 'react';

import { useAppContext } from '../../../../../../context';
import { useCanStartInstaller } from './useCanStartInstaller';

export const useHandleOnClick = () => {
  const { appUpgrade, appUpgradeInstallerStart } = useAppContext();
  const canStartInstaller = useCanStartInstaller();

  const handleOnClick = useCallback(() => {
    if (canStartInstaller) {
      appUpgradeInstallerStart();
    } else {
      appUpgrade();
    }
  }, [appUpgrade, appUpgradeInstallerStart, canStartInstaller]);

  return handleOnClick;
};
