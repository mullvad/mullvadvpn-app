import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHasSearchedLocations() {
  const { searchedLocations } = useSelectLocationViewContext();

  const hasSearchedLocations = searchedLocations.length > 0;

  return hasSearchedLocations;
}
