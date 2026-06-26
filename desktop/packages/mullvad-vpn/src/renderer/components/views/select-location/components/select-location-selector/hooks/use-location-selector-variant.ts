import { useMultihop } from '../../../../../../features/multihop/hooks';
import { LocationSelectorVariant } from '../../../../../../lib/components/location-selector';
import { useSelectLocationSelectorContext } from '../SelectLocationSelectorContext';

export function useLocationSelectorVariant(): LocationSelectorVariant {
  const { multihop } = useMultihop();
  const { isolatedItem } = useSelectLocationSelectorContext();

  if (isolatedItem) {
    return 'primary';
  }

  if (multihop === 'always') {
    return 'secondary';
  }

  return 'primary';
}
