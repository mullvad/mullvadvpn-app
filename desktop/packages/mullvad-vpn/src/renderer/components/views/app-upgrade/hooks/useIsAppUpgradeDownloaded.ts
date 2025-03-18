import { useSuggestedUpgrade } from './useSuggestedUpgrade';

export const useIsAppUpgradeDownloaded = () => {
  const suggestedUpgrade = useSuggestedUpgrade();

  const isAppUpgradeDownloaded = suggestedUpgrade?.downloaded === true;

  return isAppUpgradeDownloaded;
};
