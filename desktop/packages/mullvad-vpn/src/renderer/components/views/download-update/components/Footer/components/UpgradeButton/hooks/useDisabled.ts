import { useAppUpgradeEvent } from '../../../../../hooks';

export const useDisabled = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  switch (appUpgradeEvent?.type) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return false;

    default:
      return true;
  }
};
