import { useEntryType } from '../../../hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useShowEntryItem() {
  const { isolatedItem } = useSelectLocationViewContext();
  const entryType = useEntryType();

  if (entryType === 'entry') {
    return isolatedItem !== 'exit';
  }

  return false;
}
