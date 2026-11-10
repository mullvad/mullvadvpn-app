import { useActiveFilters } from '../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowFilterChips() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);

  const showFilterChips = isAnyFilterActive && locationType !== 'automaticEntry';

  return showFilterChips;
}
