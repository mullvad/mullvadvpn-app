import { useAppUpgradeEventType } from './useAppUpgradeEventType';

export const useIsAppUpgradePreparing = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_INITIATED':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      return true;
    default:
      return false;
  }
};
