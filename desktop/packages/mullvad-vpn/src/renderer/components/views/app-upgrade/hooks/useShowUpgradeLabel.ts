import { useHasAppUpgradeError, useIsAppUpgradePending } from '../../../../hooks';
import { useConnectionIsBlocked } from '../../../../redux/hooks';

export const useShowUpgradeLabel = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradePending = useIsAppUpgradePending();

  const showUpgradeLabel = isBlocked || hasAppUpgradeError || isAppUpgradePending;

  return showUpgradeLabel;
};
