import { LocationType } from '../../../../../../features/locations/types';
import { useMultihop } from '../../../../../../features/multihop/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';
import { useLocationListsContext } from '../../location-lists/LocationListsContext';
import { useRelayCount } from './use-relay-count';

export function useShowAutomaticLocation() {
  const { multihop } = useMultihop();
  const { type } = useLocationListsContext();
  const { searchTerm } = useSelectLocationViewContext();
  const { visibleRelays } = useRelayCount();

  const hasSearched = searchTerm.length > 1;

  return visibleRelays > 0 && multihop === 'always' && type === LocationType.entry && !hasSearched;
}
