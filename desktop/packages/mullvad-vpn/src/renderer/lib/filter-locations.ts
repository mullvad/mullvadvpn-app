import { IpVersion, LiftedConstraint, Ownership } from '../../shared/daemon-rpc-types';
import { LocationType } from '../components/views/select-location/select-location-types';
import { IRelayLocationCountryRedux, IRelayLocationRelayRedux } from '../redux/settings/reducers';
import { IpAddress, IPv4Address, IPv6Address } from './ip';

export function filterLocationsByQuic(
  locations: IRelayLocationCountryRedux[],
  quic: boolean,
  locationType: LocationType,
  multihop: boolean,
  ipVersion: LiftedConstraint<IpVersion>,
): IRelayLocationCountryRedux[] {
  const quickOnRelay = (relay: IRelayLocationRelayRedux) =>
    relay.quic !== undefined && containsIpVersionAddr(relay.quic.addrIn, ipVersion);
  return quicFilterActive(quic, locationType, multihop)
    ? filterLocationsImpl(locations, quickOnRelay)
    : locations;
}

export function filterLocationsByLwo(
  locations: IRelayLocationCountryRedux[],
  lwo: boolean,
  locationType: LocationType,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  const lwoOnRelay = (relay: IRelayLocationRelayRedux) => relay.lwo;
  return lwoFilterActive(lwo, locationType, multihop)
    ? filterLocationsImpl(locations, lwoOnRelay)
    : locations;
}

export function quicFilterActive(quic: boolean, locationType: LocationType, multihop: boolean) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;
  return quic && isEntry;
}

export function lwoFilterActive(lwo: boolean, locationType: LocationType, multihop: boolean) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;
  return lwo && isEntry;
}

export function filterLocationsByDaita(
  locations: IRelayLocationCountryRedux[],
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  return daitaFilterActive(daita, directOnly, locationType, multihop)
    ? filterLocationsImpl(locations, (relay: IRelayLocationRelayRedux) => relay.daita)
    : locations;
}

export function daitaFilterActive(
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  multihop: boolean,
) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;
  return daita && (directOnly || multihop) && isEntry;
}

export function filterLocations(
  locations: IRelayLocationCountryRedux[],
  ownership?: Ownership,
  providers?: Array<string>,
): IRelayLocationCountryRedux[] {
  const filters = [getOwnershipFilter(ownership), getProviderFilter(providers)];

  return filters.some((filter) => filter !== undefined)
    ? filterLocationsImpl(locations, (relay) => filters.every((filter) => filter?.(relay) ?? true))
    : locations;
}

function containsIpVersionAddr(addrs: string[], version: LiftedConstraint<IpVersion>): boolean {
  if (version === 'any') {
    return addrs.length > 0;
  }
  return addrs.some((strAddr) => {
    try {
      const addr = IpAddress.fromString(strAddr);
      return (
        (addr instanceof IPv4Address && version === 'ipv4') ||
        (addr instanceof IPv6Address && version === 'ipv6')
      );
    } catch {
      return false;
    }
  });
}

function getOwnershipFilter(
  ownership?: Ownership,
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  if (ownership === undefined || ownership === Ownership.any) {
    return undefined;
  }

  const expectOwned = ownership === Ownership.mullvadOwned;
  return (relay) => relay.owned === expectOwned;
}

function getProviderFilter(
  providers?: string[],
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  return providers === undefined || providers.length === 0
    ? undefined
    : (relay) => providers.includes(relay.provider);
}

function filterLocationsImpl(
  locations: Array<IRelayLocationCountryRedux>,
  filter: (relay: IRelayLocationRelayRedux) => boolean,
): Array<IRelayLocationCountryRedux> {
  return locations
    .map((country) => ({
      ...country,
      cities: country.cities
        .map((city) => ({ ...city, relays: city.relays.filter(filter) }))
        .filter((city) => city.relays.length > 0),
    }))
    .filter((country) => country.cities.length > 0);
}
