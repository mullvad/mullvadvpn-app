import { useRelayListContext } from '../../../RelayListContext';
import { useHasSearched } from './use-has-searched';

export function useHasNoCustomListsInSearchResult() {
  const { customLists } = useRelayListContext();
  const hasSearched = useHasSearched();

  if (hasSearched && !customLists.some((list) => list.visible)) {
    return true;
  }

  return false;
}
