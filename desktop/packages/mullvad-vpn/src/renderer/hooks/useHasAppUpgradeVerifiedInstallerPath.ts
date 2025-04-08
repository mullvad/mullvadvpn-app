import { useVersionSuggestedUpgrade } from '../redux/hooks';

export const useHasAppUpgradeVerifiedInstallerPath = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  const hasVerifiedInstallerPath =
    typeof suggestedUpgrade?.verifiedInstallerPath === 'string' &&
    suggestedUpgrade.verifiedInstallerPath.length > 0;

  return hasVerifiedInstallerPath;
};
