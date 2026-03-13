import {
  type IpVersion,
  type LiftedConstraint,
  ObfuscationType,
  Ownership,
} from '../../../../shared/daemon-rpc-types';
import type { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import type { LocationType } from '../types';
import { filterLocationsByFilters } from './filter-locations-by-filters';
import { getDaitaFilter } from './get-daita-filter';
import { getLwoFilter } from './get-lwo-filter';
import { getOwnershipFilter } from './get-ownership-filter';
import { getProviderFilter } from './get-provider-filter';
import { getQuicFilter } from './get-quic-filter';

export function filterLocations({
  locations,
  ownership,
  providers,
  daita,
  directOnly,
  locationType,
  multihop,
  obfuscation,
  ipVersion,
}: {
  locations: IRelayLocationCountryRedux[];
  locationType: LocationType;
  ownership?: Ownership;
  providers?: Array<string>;
  daita: boolean;
  directOnly: boolean;
  multihop: boolean;
  obfuscation: ObfuscationType;
  ipVersion: LiftedConstraint<IpVersion>;
}): IRelayLocationCountryRedux[] {
  const ownershipFilter = getOwnershipFilter(ownership);
  const providerFilter = getProviderFilter(providers);
  const daitaFilter = getDaitaFilter(daita, directOnly, locationType, multihop);
  const lwoFilter = getLwoFilter(obfuscation === ObfuscationType.lwo, locationType, multihop);
  const quicFilter = getQuicFilter(
    obfuscation === ObfuscationType.quic,
    locationType,
    multihop,
    ipVersion,
  );
  const filters = [ownershipFilter, providerFilter, daitaFilter, lwoFilter, quicFilter];

  const anyFilterActive = filters.some((filter) => filter !== undefined);

  return anyFilterActive
    ? filterLocationsByFilters(locations, (relay) =>
        filters.every((filter) => filter?.(relay) ?? true),
      )
    : locations;
}
