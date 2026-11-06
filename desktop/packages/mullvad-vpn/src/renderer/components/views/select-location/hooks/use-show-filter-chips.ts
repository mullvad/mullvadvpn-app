import { useActiveFilters } from '../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowFilterChips() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);

  if (locationType === 'automaticEntry') {
    return false;
  }

  const showFilterChips = isAnyFilterActive;

  return showFilterChips;
}
