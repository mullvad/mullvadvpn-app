import { useSelectedEntryOrExitLocation } from '../../../../features/locations/hooks';
import { useFilterViewContext } from '../FilterViewContext';

export function useShowAutomaticLocationOverridesFiltersInfo() {
  const { locationType } = useFilterViewContext();

  const location = useSelectedEntryOrExitLocation(locationType);
  const showAutomaticLocationOverridesFiltersInfo = location === 'any';

  return showAutomaticLocationOverridesFiltersInfo;
}
