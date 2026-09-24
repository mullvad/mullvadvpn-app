import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useEntryType } from './use-entry-type';

export function useShowEntryItem() {
  const { isolatedItem } = useSelectLocationViewContext();
  const entryType = useEntryType();

  if (entryType === 'entry') {
    return isolatedItem !== 'exit';
  }

  return false;
}
