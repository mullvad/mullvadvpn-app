import { useAppUpgradeEvent } from '../../../hooks';

export const useShowCancelButton = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  switch (appUpgradeEvent?.type) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
    case 'APP_UPGRADE_EVENT_STARTING_INSTALLER':
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
      return true;

    default:
      return false;
  }
};
