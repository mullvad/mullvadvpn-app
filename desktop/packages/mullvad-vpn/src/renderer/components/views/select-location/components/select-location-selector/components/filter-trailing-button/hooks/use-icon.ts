import {
  useActiveFilters,
  useSelectedEntryOrExitLocation,
} from '../../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../../features/locations/types';
import { IconProps } from '../../../../../../../../lib/components';
import { useFilterTrailingButtonContext } from '../FilterTrailingButtonContext';

export function useIcon(): IconProps['icon'] {
  const { type } = useFilterTrailingButtonContext();
  const locationType = type === 'exit' ? LocationType.exit : LocationType.entry;

  const { isAnyLocationFilterActive } = useActiveFilters(locationType);
  const location = useSelectedEntryOrExitLocation(locationType);

  if (type === 'entryAutomatic' || location === 'any') {
    return isAnyLocationFilterActive ? 'filter-overridden-active' : 'filter';
  }

  return isAnyLocationFilterActive ? 'filter-active' : 'filter';
}
