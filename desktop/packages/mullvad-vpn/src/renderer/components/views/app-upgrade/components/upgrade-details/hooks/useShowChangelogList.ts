import { useChangelog } from './useChangelog';

export const useShowChangelogList = () => {
  const changeLog = useChangelog();

  const showChangelogList = changeLog.length > 0;

  return showChangelogList;
};
