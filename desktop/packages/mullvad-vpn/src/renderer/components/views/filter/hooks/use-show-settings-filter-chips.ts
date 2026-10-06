import { useActiveFilters } from '../../../../features/locations/hooks';
import { useFilterViewContext } from '../FilterViewContext';

export function useShowSettingsFilterChips() {
  const { locationType } = useFilterViewContext();

  const { isAnySettingsFilterActive } = useActiveFilters(locationType);
  const showSettingsFilterChips = isAnySettingsFilterActive;

  return showSettingsFilterChips;
}
