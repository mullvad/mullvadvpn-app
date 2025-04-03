import { useHasChangelog } from './useHasChangelog';

export const useShowChangelogList = () => {
  const hasChangeLog = useHasChangelog();

  const showChangelog = hasChangeLog;

  return showChangelog;
};
