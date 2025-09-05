import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';

export function useShowSearchBar() {
  const canEditSplitTunneling = useCanEditSplitTunneling();

  const showSearchBar = canEditSplitTunneling;

  return showSearchBar;
}
