import { useHasChangelog } from './useHasChangelog';

export const useShowChangelogList = () => {
  const hasChangelog = useHasChangelog();

  const showChangelogList = hasChangelog;

  return showChangelogList;
};
