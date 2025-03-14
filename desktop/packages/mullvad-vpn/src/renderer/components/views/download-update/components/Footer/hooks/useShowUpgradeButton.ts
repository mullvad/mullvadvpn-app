import { useAppUpgradeEvent } from '../../../hooks';

export const useShowUpgradeButton = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  switch (appUpgradeEvent?.type) {
    case 'APP_UPGRADE_EVENT_ABORTED':
    case 'APP_UPGRADE_EVENT_ERROR':
      return true;

    default:
      return false;
  }
};
