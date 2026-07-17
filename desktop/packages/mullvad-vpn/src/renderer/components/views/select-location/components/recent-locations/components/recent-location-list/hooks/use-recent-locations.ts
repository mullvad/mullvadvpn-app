import { LocationType } from '../../../../../../../../features/locations/types';
import { useMultihop } from '../../../../../../../../features/multihop/hooks';
import { useLocationListsContext } from '../../../../location-lists/LocationListsContext';

export function useRecentLocations() {
  const {
    type,
    recentMultihopEntryLocations,
    recentMultihopExitLocations,
    recentSinglehopLocations,
  } = useLocationListsContext();
  const { multihop } = useMultihop();
  if (multihop === 'never') {
    if (recentSinglehopLocations) {
      return recentSinglehopLocations;
    }
  } else {
    if (recentMultihopEntryLocations && type === LocationType.entry) {
      return recentMultihopEntryLocations;
    } else if (recentMultihopExitLocations && type === LocationType.exit) {
      return recentMultihopExitLocations;
    }
  }
  return [];
}
