import { useCustomListLocationContext } from '../../../CustomListLocationContext';
import { useHasSearched } from './use-has-searched';

export function useHasNoCustomListsInSearchResult() {
  const { customListLocations } = useCustomListLocationContext();
  const hasSearched = useHasSearched();

  if (hasSearched && !customListLocations.some((list) => list.visible)) {
    return true;
  }

  return false;
}
