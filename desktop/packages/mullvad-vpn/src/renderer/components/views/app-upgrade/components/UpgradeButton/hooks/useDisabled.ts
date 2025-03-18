import { useConnectionIsBlocked } from '../../../../../../redux/hooks';
import { useAppUpgradeEventType } from '../../../hooks';

export const useDisabled = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();

  if (isBlocked) {
    return true;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      return false;
    default:
      return true;
  }
};
