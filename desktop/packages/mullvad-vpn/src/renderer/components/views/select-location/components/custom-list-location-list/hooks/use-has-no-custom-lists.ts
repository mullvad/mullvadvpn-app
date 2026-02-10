import { useRelayListContext } from '../../../RelayListContext';

export function useHasCustomLists() {
  const { customLists } = useRelayListContext();

  if (customLists.length > 0) {
    return true;
  }

  return false;
}
