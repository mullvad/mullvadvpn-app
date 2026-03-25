import { LocationType } from '../../../../../../features/locations/types';
import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useHasVisibleRecentLocations() {
  const {
    locationType,
    recentMultihopEntryLocations,
    recentMultihopExitLocations,
    recentSinglehopLocations,
  } = useSelectLocationViewContext();

  const { multihop } = useMultihop();

  const hasVisibleMultihopEntryLocations =
    multihop &&
    locationType === LocationType.entry &&
    recentMultihopEntryLocations &&
    recentMultihopEntryLocations.length > 0;

  const hasVisibleMultihopExitLocations =
    multihop &&
    locationType === LocationType.exit &&
    recentMultihopExitLocations &&
    recentMultihopExitLocations.length > 0;

  const hasVisibleSinglehopLocations =
    !multihop && recentSinglehopLocations && recentSinglehopLocations.length > 0;

  return (
    hasVisibleMultihopEntryLocations ||
    hasVisibleMultihopExitLocations ||
    hasVisibleSinglehopLocations
  );
}
