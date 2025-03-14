import { useAppUpgradeEventType } from '../../../hooks';

export const useShowUpgradeButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_ABORTED':
    case 'APP_UPGRADE_EVENT_ERROR':
      return true;

    default:
      return false;
  }
};
