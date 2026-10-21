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

  const { isAnyFilterActive } = useActiveFilters(locationType);
  const location = useSelectedEntryOrExitLocation(locationType);

  if (location === 'any') {
    if (isAnyFilterActive) {
      return 'filter-overridden-active';
    }

    return 'filter-overridden';
  }

  if (isAnyFilterActive) {
    return 'filter-active';
  }

  return 'filter';
}
