import { useCustomListLocationsContext } from '../../../CustomListLocationsContext';

export function useHasCustomLists() {
  const { customListLocations } = useCustomListLocationsContext();

  if (customListLocations.length > 0) {
    return true;
  }

  return false;
}
