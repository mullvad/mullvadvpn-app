import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';
import { useFilteredSplitApplications } from './use-filtered-split-applications';

export function useShowSplitApplicationList() {
  const canEditSplitTunneling = useCanEditSplitTunneling();
  const filteredSplitApplications = useFilteredSplitApplications();

  const showSplitApplicationList = canEditSplitTunneling && filteredSplitApplications.length > 0;

  return showSplitApplicationList;
}
