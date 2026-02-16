import { useCustomListLocationsContext } from '../../../CustomListLocationsContext';
import { useHasSearched } from './use-has-searched';

export function useHasNoCustomListsInSearchResult() {
  const { customListLocations } = useCustomListLocationsContext();
  const hasSearched = useHasSearched();

  if (hasSearched && customListLocations.length === 0) {
    return true;
  }

  return false;
}
