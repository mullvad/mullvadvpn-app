import { useHasAppUpgradeVerifiedInstallerPath } from '../../../../../../../../hooks';

export const useShowInstallerReady = () => {
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();

  const showInstallerReady = hasAppUpgradeVerifiedInstallerPath;

  return showInstallerReady;
};
