import { useActiveFilters } from '../../../../features/locations/hooks';
import { LocationType } from '../../../../features/locations/types';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowFilterChips() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);

  const showFilterChips = isAnyFilterActive && locationType !== LocationType.entryAutomatic;

  return showFilterChips;
}
