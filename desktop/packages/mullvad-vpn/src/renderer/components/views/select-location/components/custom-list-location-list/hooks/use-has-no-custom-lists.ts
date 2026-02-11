import { useCustomListLocationContext } from '../../../CustomListLocationContext';

export function useHasCustomLists() {
  const { customListLocations } = useCustomListLocationContext();

  if (customListLocations.length > 0) {
    return true;
  }

  return false;
}
