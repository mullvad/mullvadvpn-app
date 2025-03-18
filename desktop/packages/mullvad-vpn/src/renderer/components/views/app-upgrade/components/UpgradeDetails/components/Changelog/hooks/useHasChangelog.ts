import { useChangelog } from './useChangelog';

export const useHasChangelog = () => {
  const changelog = useChangelog();

  const hasChangeLog = changelog.length > 0;

  return hasChangeLog;
};
