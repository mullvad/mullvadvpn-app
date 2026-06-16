import type {
  IRelayLocationCountryRedux,
  RelayLocationsFilterContext,
  RelayLocationsFiltered,
} from '../../../redux/settings/reducers';
import { filterLocationsByFilters } from './filter-locations-by-filters';
import { getRelayLocationsFilteredFilter } from './get-relay-locations-filtered-filter';

export function filterLocationsByRelayLocationsFiltered(
  locations: IRelayLocationCountryRedux[],
  relayLocationsFiltered: RelayLocationsFiltered,
  context: RelayLocationsFilterContext,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  const filters = [getRelayLocationsFilteredFilter(relayLocationsFiltered, context, multihop)];

  return filterLocationsByFilters(locations, (relay) =>
    filters.every((filter) => filter?.(relay) ?? true),
  );
}
