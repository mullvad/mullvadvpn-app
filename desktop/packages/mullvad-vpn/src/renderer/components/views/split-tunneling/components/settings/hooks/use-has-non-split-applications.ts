import { useFilteredNonSplitApplications } from './use-filtered-non-split-applications';

export function useHasNonSplitApplications() {
  const filteredNonSplitApplications = useFilteredNonSplitApplications();

  const hasNonSplitApplications = filteredNonSplitApplications.length > 0;

  return hasNonSplitApplications;
}
