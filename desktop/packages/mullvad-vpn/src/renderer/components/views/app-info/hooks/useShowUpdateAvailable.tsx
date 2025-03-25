import { useVersionSuggestedUpgrade } from './useVersionSuggestedUpgrade';

export const useShowUpdateAvailable = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();
  return !!suggestedUpgrade;
};
