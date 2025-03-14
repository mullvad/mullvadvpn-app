import { AppUpgradeError } from '../../../../../../../../../shared/daemon-rpc-types';
import { useAppUpgradeEvent } from '../../../../../hooks';
import { useIsBlocked } from '../../../hooks';

export const useShowDownloadProgress = () => {
  const appUpgradeEvent = useAppUpgradeEvent();
  const isBlocked = useIsBlocked();

  if (isBlocked) {
    return false;
  }

  if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_ERROR') {
    return appUpgradeEvent.error === AppUpgradeError.verificationFailed;
  }

  return true;
};
