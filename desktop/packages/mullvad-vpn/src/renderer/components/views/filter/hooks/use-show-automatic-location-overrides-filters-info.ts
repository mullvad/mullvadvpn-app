import { useSelectedEntryOrExitLocation } from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';
import { useFilterViewContext } from '../FilterViewContext';

export function useShowAutomaticLocationOverridesFiltersInfo() {
  const { locationType } = useFilterViewContext();

  const location = useSelectedEntryOrExitLocation(
    locationType === 'exit' ? LocationType.exit : LocationType.entry,
  );
  const showAutomaticLocationOverridesFiltersInfo = location === 'any';

  return showAutomaticLocationOverridesFiltersInfo;
}
