import { useActiveFilters } from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';
import { useFilterViewContext } from '../FilterViewContext';

export function useShowSettingsFilterChips() {
  const { locationType } = useFilterViewContext();

  const { isAnySettingsFilterActive } = useActiveFilters(
    locationType === 'exit' ? LocationType.exit : LocationType.entry,
  );
  const showSettingsFilterChips = isAnySettingsFilterActive;

  return showSettingsFilterChips;
}
