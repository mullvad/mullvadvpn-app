import { useActiveFilters } from '../../../../features/locations/hooks';
import { useIsUsingMagicMultihop, useMultihop } from '../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowFilterChips() {
  const { isolatedItem, locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);
  const { multihop } = useMultihop();
  const isUsingMagicMultihop = useIsUsingMagicMultihop();

  if (locationType === 'automaticEntry') {
    return false;
  }

  // Regardless of if any filter is active, we need to show a `FilterChip`
  // informing the user which LocationSelector item is currently isolated
  // if the app has both an entry and exit. It doesn't matter whether that
  // entry is selected automatically or not, just as long as both are present.
  if (isolatedItem === 'entry' || isolatedItem === 'exit') {
    return multihop === 'always' || isUsingMagicMultihop;
  }

  return isAnyFilterActive;
}
