import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';

export const useShowCancelButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    return false;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
    case 'APP_UPGRADE_EVENT_INSTALLER_READY':
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
      return true;

    default:
      return false;
  }
};
