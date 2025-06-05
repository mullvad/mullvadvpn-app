import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useHasAppUpgradeVerifiedInstallerPath,
} from '../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../redux/hooks';

export const useDisabled = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();

  if (hasAppUpgradeVerifiedInstallerPath) {
    return false;
  }

  const disabled =
    hasAppUpgradeError || isBlocked || appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED';

  return disabled;
};
