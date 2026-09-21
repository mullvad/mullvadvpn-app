import { useMultihop } from '../../../../../../features/multihop/hooks';
import { LocationSelectorVariant } from '../../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useLocationSelectorVariant(
  automaticEntryVisible: boolean,
): LocationSelectorVariant {
  const { multihop } = useMultihop();
  const { isolatedItem } = useSelectLocationViewContext();

  if (isolatedItem) {
    return 'primary';
  }

  if (multihop === 'always') {
    return 'secondary';
  } else if (multihop === 'when-needed' && automaticEntryVisible) {
    return 'secondary';
  }

  return 'primary';
}
