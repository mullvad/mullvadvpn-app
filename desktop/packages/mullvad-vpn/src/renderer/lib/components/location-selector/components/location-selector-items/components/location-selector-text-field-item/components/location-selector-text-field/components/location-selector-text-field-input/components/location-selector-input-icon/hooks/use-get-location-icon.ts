import type { icons } from '../../../../../../../../../../../../icon/types';
import { useTextFieldContext } from '../../../../../../../../../../../../text-field';
import type { LocationSelectorItemType } from '../../../../../../../../../types';
import { useLocationSelectorTextFieldItemContext } from '../../../../../../../LocationSelectorTextFieldItemContext';

export function useGetLocationIcon(type: LocationSelectorItemType): keyof typeof icons {
  const { focusInsideTextField } = useLocationSelectorTextFieldItemContext();
  const { invalid } = useTextFieldContext();

  if (focusInsideTextField) {
    return 'search';
  }

  if (invalid) {
    return 'alert';
  }

  switch (type) {
    case 'entry':
      return 'location-add';
    case 'entryAutomatic':
      return 'magic-multihop';
    case 'exit':
      return 'location';
    default:
      return type satisfies never;
  }
}
