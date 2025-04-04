import { useIsAppUpgradeInProgress } from '../../../../hooks';

export const useShowCancelButton = () => {
  const isAppUpgradeInProgress = useIsAppUpgradeInProgress();

  const showCancelButton = isAppUpgradeInProgress;

  return showCancelButton;
};
