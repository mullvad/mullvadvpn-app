import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../redux/hooks';

export const useDisabled = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  const disabled =
    hasAppUpgradeError || isBlocked || appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED';

  return disabled;
};
