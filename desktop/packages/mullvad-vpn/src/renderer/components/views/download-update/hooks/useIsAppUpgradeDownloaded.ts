import { useSuggestedUpgrade } from './useSuggestedUpgrade';

export const useIsAppUpgradeDownloaded = () => {
  const suggestedUpgrade = useSuggestedUpgrade();

  const appUpgradeDownloaded = suggestedUpgrade?.downloaded;

  return appUpgradeDownloaded;
};
