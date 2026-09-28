import { useActiveFilters } from '../../../../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../../../../features/locations/types';
import { IconProps } from '../../../../../../../../../../lib/components';

export function useIcon(): IconProps['icon'] {
  const { isAnyLocationFilterActive } = useActiveFilters(LocationType.entry);

  if (isAnyLocationFilterActive) {
    return 'filter-overridden-active';
  }

  return 'filter-overridden';
}
