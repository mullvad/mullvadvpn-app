import { useAppUpgradeEvent } from '../../../../../hooks';
import { useIsBlocked } from '../../../hooks';

export const useDisabled = () => {
  const appUpgradeEvent = useAppUpgradeEvent();
  const isBlocked = useIsBlocked();

  if (isBlocked) {
    return true;
  }

  switch (appUpgradeEvent?.type) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return false;

    default:
      return true;
  }
};
