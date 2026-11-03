import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useShowEntryFilterChip() {
  const { isolatedItem } = useSelectLocationViewContext();
  const { multihop } = useMultihop();

  const showEntryFilterChip = multihop === 'always' && isolatedItem === 'entry';

  return showEntryFilterChip;
}
