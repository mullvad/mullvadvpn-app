import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';

export const useShowCancelButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    return false;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      return true;
    default:
      return false;
  }
};
