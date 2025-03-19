import { useAppUpgradeEventType, useIsAppUpgradeDownloaded } from '../../../hooks';

export const useCanStartInstaller = () => {
  const isAppUpgradeDownloaded = useIsAppUpgradeDownloaded();
  const appUpgradeEventType = useAppUpgradeEventType();

  const canStartInstaller =
    isAppUpgradeDownloaded || appUpgradeEventType === 'APP_UPGRADE_EVENT_INSTALLER_READY';

  return canStartInstaller;
};
