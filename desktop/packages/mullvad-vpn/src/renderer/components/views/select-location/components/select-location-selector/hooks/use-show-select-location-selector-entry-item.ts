import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationSelectorContext } from '../SelectLocationSelectorContext';
import { useIsEntryAutomatic } from './use-is-entry-automatic';

export function useShowSelectLocationSelectorEntryItem() {
  const { multihop } = useMultihop();
  const { isolatedItem } = useSelectLocationSelectorContext();
  const isEntryAutomatic = useIsEntryAutomatic();

  if (multihop === 'when-needed') {
    return isEntryAutomatic;
  }

  if (multihop === 'always') {
    return isolatedItem !== 'exit';
  }

  return false;
}
