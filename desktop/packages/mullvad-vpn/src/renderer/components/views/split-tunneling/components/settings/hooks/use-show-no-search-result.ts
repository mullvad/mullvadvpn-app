import { useSettingsContext } from '../SettingsContext';
import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';
import { useHasNonSplitApplications } from './use-has-non-split-applications';
import { useHasSplitApplications } from './use-has-split-applications';

export function useShowNoSearchResult() {
  const { searchTerm } = useSettingsContext();
  const canEditSplitTunneling = useCanEditSplitTunneling();
  const hasNonSplitApplications = useHasNonSplitApplications();
  const hasSplitApplications = useHasSplitApplications();

  const showNoSearchResult =
    canEditSplitTunneling && searchTerm !== '' && !hasSplitApplications && !hasNonSplitApplications;

  return showNoSearchResult;
}
