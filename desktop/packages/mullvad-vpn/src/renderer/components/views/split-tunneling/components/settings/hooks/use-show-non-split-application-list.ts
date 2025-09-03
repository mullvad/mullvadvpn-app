import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';
import { useFilteredNonSplitApplications } from './use-filtered-non-split-applications';

export function useShowNonSplitApplicationList() {
  const canEditSplitTunneling = useCanEditSplitTunneling();
  const filteredNonSplitApplications = useFilteredNonSplitApplications();

  const showNonSplitApplicationList =
    canEditSplitTunneling &&
    (!filteredNonSplitApplications || filteredNonSplitApplications.length > 0);

  return showNonSplitApplicationList;
}
