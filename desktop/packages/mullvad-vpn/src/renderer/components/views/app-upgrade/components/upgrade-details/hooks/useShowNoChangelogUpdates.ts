import { useHasChangelog } from './useHasChangelog';

export const useShowNoChangelogUpdates = () => {
  const hasChangelog = useHasChangelog();

  const showNoChangelogUpdates = !hasChangelog;

  return showNoChangelogUpdates;
};
