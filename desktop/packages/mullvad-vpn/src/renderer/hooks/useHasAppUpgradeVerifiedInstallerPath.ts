import { useVersionSuggestedUpgrade } from '../redux/hooks';

export const useHasAppUpgradeVerifiedInstallerPath = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  const hasVerifiedInstallerPath = suggestedUpgrade?.verifiedInstallerPath !== undefined;

  return hasVerifiedInstallerPath;
};
