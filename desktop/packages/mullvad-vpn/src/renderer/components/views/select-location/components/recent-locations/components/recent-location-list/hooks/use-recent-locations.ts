import { LocationType } from '../../../../../../../../features/locations/types';
import { useLocationListsContext } from '../../../../location-lists/LocationListsContext';

export function useRecentLocations() {
  const { type, recentEntryLocations, recentExitLocations } = useLocationListsContext();
  if (recentEntryLocations && type === LocationType.entry) {
    return recentEntryLocations;
  } else if (recentExitLocations && type === LocationType.exit) {
    return recentExitLocations;
  }
  return [];
}
