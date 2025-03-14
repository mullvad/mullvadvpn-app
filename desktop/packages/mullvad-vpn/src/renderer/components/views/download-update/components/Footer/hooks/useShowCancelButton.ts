import { useAppUpgradeEventType } from '../../../hooks';

export const useShowCancelButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
    case 'APP_UPGRADE_EVENT_STARTING_INSTALLER':
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
      return true;

    default:
      return false;
  }
};
