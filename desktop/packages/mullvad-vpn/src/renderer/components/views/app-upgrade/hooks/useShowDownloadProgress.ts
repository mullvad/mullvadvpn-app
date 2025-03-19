import { AppUpgradeError } from '../../../../../shared/daemon-rpc-types';
import { useAppUpgradeError } from './useAppUpgradeError';
import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useIsBlocked } from './useIsBlocked';

export const useShowDownloadProgress = () => {
  const appUpgradeError = useAppUpgradeError();
  const appUpgradeEventType = useAppUpgradeEventType();
  const isBlocked = useIsBlocked();

  if (!isBlocked) {
    switch (appUpgradeEventType) {
      case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
      case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      case 'APP_UPGRADE_EVENT_INSTALLER_READY':
      case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
        return true;
      default:
        break;
    }

    if (appUpgradeError === AppUpgradeError.verificationFailed) {
      return true;
    }
  }

  return false;
};
