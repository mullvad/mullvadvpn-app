import { MultihopMode } from '../../../../shared/daemon-rpc-types';
import type {
  IRelayLocationCountryRedux,
  RelayLocationsFilterContext,
  RelayLocationsFiltered,
} from '../../../redux/settings/reducers';
import { filterCountries } from './filter-countries';
import { getRelayLocationsFilteredFilter } from './get-relay-locations-filtered-filter';

export function filterCountriesByRelayLocationsFiltered(
  locations: IRelayLocationCountryRedux[],
  relayLocationsFiltered: RelayLocationsFiltered,
  context: RelayLocationsFilterContext,
  multihop: MultihopMode,
): IRelayLocationCountryRedux[] {
  const filters = [getRelayLocationsFilteredFilter(relayLocationsFiltered, context, multihop)];

  return filterCountries(locations, (relay) => filters.every((filter) => filter?.(relay) ?? true));
}
