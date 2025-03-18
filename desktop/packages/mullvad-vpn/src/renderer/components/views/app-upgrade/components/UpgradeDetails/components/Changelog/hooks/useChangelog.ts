import { useVersionSuggestedUpgrade } from '../../../../../../../../redux/hooks';

export const useChangelog = () => {
  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  if (suggestedUpgrade?.changelog) {
    const changelog = suggestedUpgrade.changelog.split('\n');

    return changelog;
  }

  return [];
};
