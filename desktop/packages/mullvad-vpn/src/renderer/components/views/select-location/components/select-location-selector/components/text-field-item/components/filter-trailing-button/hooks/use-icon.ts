import { useActiveFilters } from '../../../../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../../../../features/locations/types';
import { IconProps } from '../../../../../../../../../../lib/components';
import { useTextFieldItemContext } from '../../../TextFieldItemContext';

export function useIcon(): IconProps['icon'] {
  const { id } = useTextFieldItemContext();
  const locationType = id === 'exit' ? LocationType.exit : LocationType.entry;
  const { isAnyLocationFilterActive } = useActiveFilters(locationType);

  if (id == 'entryAutomatic') {
    return isAnyLocationFilterActive ? 'filter-overridden-active' : 'filter-overridden';
  }

  return isAnyLocationFilterActive ? 'filter-active' : 'filter';
}
