import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHasSearchedLocations() {
  const { countryLocations } = useSelectLocationViewContext();

  const hasSearchedLocations = countryLocations.length > 0;

  return hasSearchedLocations;
}
