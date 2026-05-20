import type { icons } from '../../../../../../../../../../../../icon/types';
import type { LocationSelectorItemType } from '../../../../../../../LocationSelectorItem';
import { useLocationSelectorItemContext } from '../../../../../../../LocationSelectorItemContext';

export function useGetLocationIcon(type: LocationSelectorItemType): keyof typeof icons {
  const { inputFocused } = useLocationSelectorItemContext();
  if (inputFocused) {
    return 'search';
  }

  switch (type) {
    case 'entry':
      return 'location-add';
    case 'entryAutomatic':
      return 'magic-multihop';
    case 'exit':
      return 'location';
    default:
      return 'location';
  }
}
