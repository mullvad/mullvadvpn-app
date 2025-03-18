import { useSuggestedUpgrade } from './useSuggestedUpgrade';

export const useIsAppUpgradeInstallerReady = () => {
  const suggestedUpgrade = useSuggestedUpgrade();

  const isAppUpgradeInstallerReady = suggestedUpgrade?.verifiedInstallerPath !== undefined;

  return isAppUpgradeInstallerReady;
};
