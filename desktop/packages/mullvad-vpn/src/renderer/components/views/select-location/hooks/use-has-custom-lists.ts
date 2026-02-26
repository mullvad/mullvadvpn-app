import { useCustomListLocationsContext } from '../CustomListLocationsContext';

export function useHasCustomLists() {
  const { customListLocations } = useCustomListLocationsContext();

  const hasCustomLists = customListLocations.length > 0;

  return hasCustomLists;
}
