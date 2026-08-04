import { LocationType } from '../../../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';

export function useRecentLocations() {
  const { locationType, recentEntryLocations, recentExitLocations } =
    useSelectLocationViewContext();
  if (recentEntryLocations && locationType === LocationType.entry) {
    return recentEntryLocations;
  } else if (recentExitLocations && locationType === LocationType.exit) {
    return recentExitLocations;
  }

  return [];
}
