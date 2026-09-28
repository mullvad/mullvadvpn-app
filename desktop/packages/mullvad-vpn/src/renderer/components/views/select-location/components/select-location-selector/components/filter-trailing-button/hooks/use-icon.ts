import {
  useActiveFilters,
  useSelectedEntryOrExitLocation,
} from '../../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../../features/locations/types';
import { IconProps } from '../../../../../../../../lib/components';
import { useEntryType } from '../../../../../hooks';
import { useFilterTrailingButtonContext } from '../FilterTrailingButtonContext';

export function useIcon(): IconProps['icon'] {
  const { id } = useFilterTrailingButtonContext();
  const locationType = id === 'exit' ? LocationType.exit : LocationType.entry;

  const { isAnyLocationFilterActive } = useActiveFilters(locationType);
  const location = useSelectedEntryOrExitLocation(locationType);
  const entryType = useEntryType();

  if (location === 'any' || entryType === 'entryAutomatic') {
    return isAnyLocationFilterActive ? 'filter-overridden-active' : 'filter';
  }

  return isAnyLocationFilterActive ? 'filter-active' : 'filter';
}
