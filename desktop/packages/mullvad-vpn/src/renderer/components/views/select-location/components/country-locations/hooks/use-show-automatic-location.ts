import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';
import { useLocationListsContext } from '../../location-lists/LocationListsContext';

export function useShowAutomaticLocation() {
  const { multihop } = useMultihop();
  const { type } = useLocationListsContext();
  const { searchTerm } = useSelectLocationViewContext();

  const hasSearched = searchTerm.length > 1;

  return multihop === 'always' && type === 'entry' && !hasSearched;
}
