import { useLocationListsContext } from '../LocationListsContext';

export function useHasCustomLists() {
  const { customListLocations } = useLocationListsContext();

  const hasCustomLists = customListLocations.length > 0;

  return hasCustomLists;
}
