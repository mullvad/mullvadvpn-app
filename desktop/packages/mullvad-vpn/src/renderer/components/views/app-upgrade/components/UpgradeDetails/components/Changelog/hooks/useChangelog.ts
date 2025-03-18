import { useSuggestedUpgrade } from '../../../../../hooks';

export const useChangelog = () => {
  const suggestedUpgrade = useSuggestedUpgrade();

  if (suggestedUpgrade?.changelog) {
    const changelog = suggestedUpgrade.changelog.split('\n');

    return changelog;
  }

  return [];
};
