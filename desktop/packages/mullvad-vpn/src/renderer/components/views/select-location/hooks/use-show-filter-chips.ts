import { useActiveFilters } from '../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useEntryType } from './use-entry-type';

export function useShowFilterChips() {
  const { locationType } = useSelectLocationViewContext();
  const entryType = useEntryType();

  const { isAnyFilterActive } = useActiveFilters(locationType);
  return isAnyFilterActive && entryType !== 'entryAutomatic';
}
