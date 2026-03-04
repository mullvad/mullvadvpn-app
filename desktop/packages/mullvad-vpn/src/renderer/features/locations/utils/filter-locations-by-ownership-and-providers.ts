import { Ownership } from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import { filterLocationsByFilters } from './filter-locations-by-filters';
import { getOwnershipFilter } from './get-ownership-filter';
import { getProviderFilter } from './get-provider-filter';

export function filterLocationsByOwnershipAndProviders(
  locations: IRelayLocationCountryRedux[],
  ownership?: Ownership,
  providers?: Array<string>,
): IRelayLocationCountryRedux[] {
  const filters = [getOwnershipFilter(ownership), getProviderFilter(providers)];

  return filters.some((filter) => filter !== undefined)
    ? filterLocationsByFilters(locations, (relay) =>
        filters.every((filter) => filter?.(relay) ?? true),
      )
    : locations;
}
