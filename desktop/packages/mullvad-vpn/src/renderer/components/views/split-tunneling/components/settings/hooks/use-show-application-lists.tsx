import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';
import { useHasNonSplitApplications } from './use-has-non-split-applications';
import { useHasSplitApplications } from './use-has-split-applications';

export function useShowApplicationLists() {
  const canEditSplitTunneling = useCanEditSplitTunneling();
  const hasNonSplitApplications = useHasNonSplitApplications();
  const hasSplitApplications = useHasSplitApplications();

  const showApplicationLists =
    canEditSplitTunneling && (hasSplitApplications || hasNonSplitApplications);

  return showApplicationLists;
}
