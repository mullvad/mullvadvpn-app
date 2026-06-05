import { RelayPartitionsContext } from '../../../redux/settings/reducers';
import { useSelector } from '../../../redux/store';
import { useMultihop } from '../../multihop/hooks';
import { LocationType } from '../types';
import { filterLocationsByFilters, getRelayPartitionsFilter } from '../utils';

export function useFilterCountryLocations(locationType: LocationType) {
  const locations = useSelector((state) => state.settings.relayLocations);
  const relayPartitions = useSelector((state) => state.settings.relayPartitions);
  const { multihop } = useMultihop();

  const context: RelayPartitionsContext = locationType === LocationType.entry ? 'entry' : 'exit';
  const filters = [getRelayPartitionsFilter(relayPartitions, context, multihop)];

  return filterLocationsByFilters(locations, (relay) =>
    filters.every((filter) => filter?.(relay) ?? true),
  );
}
