import { useSuggestedUpgrade } from '../../../hooks';

export const useChangelog = () => {
  const suggestedUpgrade = useSuggestedUpgrade();

  if (suggestedUpgrade) {
    if (suggestedUpgrade.changelog) {
      const changeLog = suggestedUpgrade.changelog.split('\n');

      return changeLog;
    }
  }

  return [];
};
