import {
  useHasAppUpgradeVerifiedInstallerPath,
  useIsAppUpgradeInProgress,
} from '../../../../hooks';

export const useShowCancelButton = () => {
  const isAppUpgradeInProgress = useIsAppUpgradeInProgress();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();

  const showCancelButton = isAppUpgradeInProgress && !hasAppUpgradeVerifiedInstallerPath;

  return showCancelButton;
};
