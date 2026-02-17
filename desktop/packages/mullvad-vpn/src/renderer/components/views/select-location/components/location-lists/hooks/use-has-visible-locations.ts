import { useLocationsContext } from '../../../LocationsContext';

export function useHasSearchedLocations() {
  const { searchedLocations } = useLocationsContext();

  const hasSearchedLocations = searchedLocations.length > 0;

  return hasSearchedLocations;
}
