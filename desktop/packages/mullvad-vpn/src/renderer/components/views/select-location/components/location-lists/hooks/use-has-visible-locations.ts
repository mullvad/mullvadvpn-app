import { useLocationListsContext } from '../LocationListsContext';

export function useHasSearchedLocations() {
  const { countryLocations } = useLocationListsContext();

  const hasSearchedLocations = countryLocations.length > 0;

  return hasSearchedLocations;
}
