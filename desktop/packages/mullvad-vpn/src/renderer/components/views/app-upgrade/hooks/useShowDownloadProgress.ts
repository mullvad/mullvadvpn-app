import {
  useHasAppUpgradeError,
  useHasAppUpgradeInitiated,
  useIsAppUpgradePending,
} from '../../../../hooks';
import { useAppUpgradeError, useConnectionIsBlocked } from '../../../../redux/hooks';

export const useShowDownloadProgress = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const { error } = useAppUpgradeError();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradePending = useIsAppUpgradePending();

  if (isBlocked && hasAppUpgradeInitiated) {
    return true;
  }

  if (hasAppUpgradeError) {
    if (
      error === 'START_INSTALLER_AUTOMATIC_FAILED' ||
      error === 'START_INSTALLER_FAILED' ||
      error === 'VERIFICATION_FAILED'
    ) {
      return true;
    }

    return false;
  }

  return isAppUpgradePending;
};
