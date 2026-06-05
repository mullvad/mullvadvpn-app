import type {
  IRelayLocationCountryRedux,
  RelayPartitions,
  RelayPartitionsContext,
} from '../../../redux/settings/reducers';
import { filterLocationsByFilters } from './filter-locations-by-filters';
import { getRelayPartitionsFilter } from './get-relay-partitions-filter';

export function filterLocationsByRelayPartitions(
  locations: IRelayLocationCountryRedux[],
  relayPartitions: RelayPartitions,
  context: RelayPartitionsContext,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  const filters = [getRelayPartitionsFilter(relayPartitions, context, multihop)];

  return filterLocationsByFilters(locations, (relay) =>
    filters.every((filter) => filter?.(relay) ?? true),
  );
}
