import { useActiveFilters } from '../../../../features/locations/hooks';
import { useMultihop } from '../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowFilterChips() {
  const { isolatedItem, locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);
  const { multihop } = useMultihop();

  const showFilterChips =
    (multihop === 'always' && isolatedItem !== 'automaticEntry') ||
    (isAnyFilterActive && locationType !== 'automaticEntry');

  return showFilterChips;
}
