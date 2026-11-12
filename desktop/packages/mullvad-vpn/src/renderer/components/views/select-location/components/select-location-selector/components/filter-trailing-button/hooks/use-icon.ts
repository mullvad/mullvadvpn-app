import {
  useActiveFilters,
  useSelectedEntryOrExitLocation,
} from '../../../../../../../../features/locations/hooks';
import { IconProps } from '../../../../../../../../lib/components';
import { useFilterTrailingButtonContext } from '../FilterTrailingButtonContext';

export function useIcon(): IconProps['icon'] {
  const { type } = useFilterTrailingButtonContext();
  const locationType = type === 'exit' ? 'exit' : 'entry';

  const { isAnyFilterActive } = useActiveFilters(locationType);
  const location = useSelectedEntryOrExitLocation(locationType);

  const isAutomaticLocation = location === 'any';
  const isAutomaticMultihopEntry = type === 'automaticEntry';

  if (isAutomaticLocation || isAutomaticMultihopEntry) {
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
