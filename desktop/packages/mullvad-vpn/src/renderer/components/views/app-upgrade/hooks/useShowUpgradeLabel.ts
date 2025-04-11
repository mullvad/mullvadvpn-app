import {
  useHasAppUpgradeError,
  useIsAppUpgradePending,
  useShouldAppUpgradeInstallManually,
} from '../../../../hooks';
import { useConnectionIsBlocked } from '../../../../redux/hooks';

export const useShowUpgradeLabel = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradePending = useIsAppUpgradePending();
  const shouldAppUpgradeInstallManually = useShouldAppUpgradeInstallManually();

  const showUpgradeLabel =
    isBlocked || hasAppUpgradeError || isAppUpgradePending || shouldAppUpgradeInstallManually;

  return showUpgradeLabel;
};
