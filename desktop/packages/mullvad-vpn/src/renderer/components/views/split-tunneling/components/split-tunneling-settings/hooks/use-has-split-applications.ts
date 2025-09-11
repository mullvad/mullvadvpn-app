import { useFilteredSplitApplications } from './use-filtered-split-applications';

export function useHasSplitApplications() {
  const filteredSplitApplications = useFilteredSplitApplications();

  const hasSplitApplications = filteredSplitApplications.length > 0;

  return hasSplitApplications;
}
