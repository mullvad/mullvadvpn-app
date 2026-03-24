import { LocationType } from '../../../../../../features/locations/types';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHasVisibleRecentLocations() {
  const { locationType, recentLocations } = useSelectLocationViewContext();

  const hasVisibleEntryRecentLocations =
    locationType === LocationType.entry && recentLocations.entry.length > 0;
  const hasVisibleExitRecentLocations =
    locationType === LocationType.exit && recentLocations.exit.length > 0;

  return hasVisibleEntryRecentLocations || hasVisibleExitRecentLocations;
}
