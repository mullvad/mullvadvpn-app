import { useHasAppUpgradeInitiated, useIsAppUpgradePending } from '../../../../hooks';
import { useAppUpgradeError, useConnectionIsBlocked } from '../../../../redux/hooks';

export const useShowDownloadProgress = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const { appUpgradeError } = useAppUpgradeError();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();
  const isAppUpgradePending = useIsAppUpgradePending();

  if (isBlocked && hasAppUpgradeInitiated) {
    return true;
  }

  if (appUpgradeError === 'VERIFICATION_FAILED') {
    return true;
  }

  return isAppUpgradePending;
};
