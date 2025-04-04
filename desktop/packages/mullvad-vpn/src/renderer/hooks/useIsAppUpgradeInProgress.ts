import { useHasAppUpgradeError } from './useHasAppUpgradeError';
import { useIsAppUpgradePending } from './useIsAppUpgradePending';

export const useIsAppUpgradeInProgress = () => {
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradePending = useIsAppUpgradePending();

  const isAppUpgradeInProgress = isAppUpgradePending && !hasAppUpgradeError;

  return isAppUpgradeInProgress;
};
