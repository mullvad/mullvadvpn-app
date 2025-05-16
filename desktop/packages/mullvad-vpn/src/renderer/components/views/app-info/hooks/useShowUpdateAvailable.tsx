import { useVersionSuggestedUpgrade } from '../../../../redux/version/hooks/useVersionSuggestedUpgrade';

export const useShowUpdateAvailable = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();
  return !!suggestedUpgrade;
};
