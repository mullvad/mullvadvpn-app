import { useHasAppUpgradeError, useIsAppUpgradePending } from '../../../../hooks';
import { useConnectionIsBlocked } from '../../../../redux/hooks';

export const useShowUpgradeLabel = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradePending = useIsAppUpgradePending();

  if (isBlocked || hasAppUpgradeError) {
    return true;
  }

  const showUpgradeLabel = isAppUpgradePending;

  return showUpgradeLabel;
};
