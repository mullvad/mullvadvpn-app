import { useSettingsContext } from '../SettingsContext';
import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';
import { useShowNonSplitApplicationList } from './use-show-non-split-application-list';
import { useShowSplitApplicationList } from './use-show-split-application-list';

export function useShowNoSearchResult() {
  const { searchTerm } = useSettingsContext();
  const canEditSplitTunneling = useCanEditSplitTunneling();
  const showNonSplitApplicationList = useShowNonSplitApplicationList();
  const showSplitApplicationList = useShowSplitApplicationList();

  const showNoSearchResult =
    canEditSplitTunneling &&
    searchTerm !== '' &&
    !showSplitApplicationList &&
    !showNonSplitApplicationList;

  return showNoSearchResult;
}
