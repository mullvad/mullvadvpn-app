import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../redux/hooks';

export const useDisabled = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const { isBlocked } = useConnectionIsBlocked();
  if (hasAppUpgradeError || isBlocked || appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED') {
    return true;
  }

  return false;
};
