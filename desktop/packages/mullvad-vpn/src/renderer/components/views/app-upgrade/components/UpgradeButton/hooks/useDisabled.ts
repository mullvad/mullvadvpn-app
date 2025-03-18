import { useAppUpgradeEventType, useIsBlocked } from '../../../hooks';

export const useDisabled = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const isBlocked = useIsBlocked();

  if (isBlocked) {
    return true;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return false;

    default:
      return true;
  }
};
