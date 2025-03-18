import { useAppUpgradeError, useIsAppUpgradeInstallerReady } from '../../../hooks';

export const useCanStartInstaller = () => {
  const appUpgradeError = useAppUpgradeError();
  const isAppUpgradeInstallerReady = useIsAppUpgradeInstallerReady();

  const canStartInstaller =
    isAppUpgradeInstallerReady && appUpgradeError === 'START_INSTALLER_FAILED';

  return canStartInstaller;
};
