import { useActiveFilters } from '../../../../features/locations/hooks';
import { useMultihop } from '../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowFilterChips() {
  const { isolatedItem, locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);
  const { multihop } = useMultihop();

  if (locationType === 'automaticEntry') {
    return false;
  }

  const showIsolatedItemLocationTypeFilterChip =
    multihop === 'always' && (isolatedItem === 'entry' || isolatedItem === 'exit');

  const showFilterChips = isAnyFilterActive || showIsolatedItemLocationTypeFilterChip;

  return showFilterChips;
}
