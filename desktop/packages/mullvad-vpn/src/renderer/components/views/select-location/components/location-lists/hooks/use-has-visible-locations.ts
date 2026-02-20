import { useLocationsContext } from '../../../LocationsContext';

export function useHasSearchedLocations() {
  const { searchedLocations } = useLocationsContext();

  if (searchedLocations.length === 0) {
    return false;
  }

  return true;
}
