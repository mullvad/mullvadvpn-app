import { useAppUpgradeEventType } from '../../../hooks';

export const useDisabled = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      return false;
    default:
      return true;
  }
};
