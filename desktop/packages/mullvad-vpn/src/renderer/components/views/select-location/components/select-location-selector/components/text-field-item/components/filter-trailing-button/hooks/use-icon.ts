import { useActiveFilters } from '../../../../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../../../../features/locations/types';
import { IconProps } from '../../../../../../../../../../lib/components';
import { useTextFieldItemContext } from '../../../TextFieldItemContext';

export function useIcon(): IconProps['icon'] {
  const { id } = useTextFieldItemContext();
  const locationType = id === 'exit' ? LocationType.exit : LocationType.entry;
  const { isAnyListFilterActive } = useActiveFilters(locationType);

  if (id == 'entryAutomatic') {
    return isAnyListFilterActive ? 'filter-overridden-active' : 'filter-overridden';
  }

  return isAnyListFilterActive ? 'filter-active' : 'filter';
}
