import { useMultihop } from '../../../../../../features/multihop/hooks';
import { LocationSelectorVariant } from '../../../../../../lib/components/location-selector';
import { useShowAutomaticEntryItem } from '../../../hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useLocationSelectorVariant(): LocationSelectorVariant {
  const { multihop } = useMultihop();
  const { isolatedItem } = useSelectLocationViewContext();

  const showAutomaticEntryItem = useShowAutomaticEntryItem();

  if (isolatedItem) {
    return 'primary';
  }

  if (multihop === 'always') {
    return 'secondary';
  } else if (multihop === 'when-needed' && showAutomaticEntryItem) {
    return 'secondary';
  }

  return 'primary';
}
