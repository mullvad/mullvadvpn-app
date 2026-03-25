import { LocationType } from '../../../../../../features/locations/types';
import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useRecentLocations() {
  const {
    locationType,
    recentMultihopEntryLocations,
    recentMultihopExitLocations,
    recentSinglehopLocations,
  } = useSelectLocationViewContext();
  const { multihop } = useMultihop();
  if (!multihop) {
    if (recentSinglehopLocations) {
      return recentSinglehopLocations;
    }
  } else {
    if (recentMultihopEntryLocations && locationType === LocationType.entry) {
      return recentMultihopEntryLocations;
    } else if (recentMultihopExitLocations && locationType === LocationType.exit) {
      return recentMultihopExitLocations;
    }
  }
  return [];
}
