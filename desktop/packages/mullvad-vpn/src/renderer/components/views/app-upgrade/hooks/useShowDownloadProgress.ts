import {
  useHasAppUpgradeError,
  useHasAppUpgradeInitiated,
  useIsAppUpgradePending,
} from '../../../../hooks';
import { useAppUpgradeError, useConnectionIsBlocked } from '../../../../redux/hooks';

export const useShowDownloadProgress = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const { appUpgradeError } = useAppUpgradeError();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradePending = useIsAppUpgradePending();

  if (isBlocked && hasAppUpgradeInitiated) {
    return true;
  }

  if (hasAppUpgradeError) {
    if (
      appUpgradeError === 'START_INSTALLER_AUTOMATIC_FAILED' ||
      appUpgradeError === 'START_INSTALLER_FAILED' ||
      appUpgradeError === 'VERIFICATION_FAILED'
    ) {
      return true;
    }

    return false;
  }

  return isAppUpgradePending;
};
