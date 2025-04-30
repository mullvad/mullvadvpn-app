import { useVersionSuggestedUpgrade } from '../../../../redux/version/hooks/useVersionSuggestedUpgrade';

export const useShowUpdateAvailable = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  console.log('suggestedUpgrade', suggestedUpgrade);

  return !!suggestedUpgrade;
};
