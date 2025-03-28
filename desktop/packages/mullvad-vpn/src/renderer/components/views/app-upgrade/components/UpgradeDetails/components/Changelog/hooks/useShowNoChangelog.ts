import { useHasChangelog } from './useHasChangelog';

export const useShowNoChangelog = () => {
  const hasChangelog = useHasChangelog();

  const showNoChangelog = !hasChangelog;

  return showNoChangelog;
};
