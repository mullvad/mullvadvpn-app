import {
  Ownership,
  RelayEndpointType,
  RelayLocation,
  TunnelProtocol,
} from '../../shared/daemon-rpc-types';
import { relayLocations } from '../../shared/gettext';
import {
  LocationType,
  RelayLocationCityWithVisibility,
  RelayLocationCountryWithVisibility,
  RelayLocationRelayWithVisibility,
  SpecialLocation,
} from '../components/select-location/select-location-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../redux/settings/reducers';

export enum EndpointType {
  any,
  entry,
  exit,
}

export function filterLocationsByEndPointType(
  locations: IRelayLocationCountryRedux[],
  endpointType: EndpointType,
  tunnelProtocol: TunnelProtocol,
): IRelayLocationCountryRedux[] {
  return filterLocationsImpl(locations, getTunnelProtocolFilter(endpointType, tunnelProtocol));
}

export function filterLocationsByDaita(
  locations: IRelayLocationCountryRedux[],
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  tunnelProtocol: TunnelProtocol,
  multihop: boolean,
): IRelayLocationCountryRedux[] {
  return daitaFilterActive(daita, directOnly, locationType, tunnelProtocol, multihop)
    ? filterLocationsImpl(locations, (relay: IRelayLocationRelayRedux) => relay.daita)
    : locations;
}

export function daitaFilterActive(
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  tunnelProtocol: TunnelProtocol,
  multihop: boolean,
) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;
  return daita && (directOnly || multihop) && isEntry && tunnelProtocol !== 'openvpn';
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

function getTunnelProtocolFilter(
  endpointType: EndpointType,
  tunnelProtocol: TunnelProtocol,
): (relay: IRelayLocationRelayRedux) => boolean {
  const endpointTypes: Array<RelayEndpointType> = [];
  if (endpointType !== EndpointType.exit && tunnelProtocol === 'openvpn') {
    endpointTypes.push('bridge');
  } else {
    endpointTypes.push(tunnelProtocol);
  }

  return (relay) => endpointTypes.includes(relay.endpointType);
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

export function searchForLocations(
  countries: Array<IRelayLocationCountryRedux>,
  searchTerm: string,
): Array<RelayLocationCountryWithVisibility> {
  return countries.map((country) => {
    const match =
      searchTerm === '' ||
      searchMatch(searchTerm, country.code) ||
      searchMatch(searchTerm, relayLocations.gettext(country.name));
    const cities = searchCities(country.cities, searchTerm, match);
    const expanded = cities.some((city) => city.visible);
    return { ...country, cities: cities, visible: expanded || match };
  });
}

function searchCities(
  cities: Array<IRelayLocationCityRedux>,
  searchTerm: string,
  countryMatch: boolean,
): Array<RelayLocationCityWithVisibility> {
  return cities.map((city) => {
    const match =
      searchTerm === '' ||
      countryMatch ||
      searchMatch(searchTerm, city.code) ||
      searchMatch(searchTerm, relayLocations.gettext(city.name));
    const relays = searchRelays(city.relays, searchTerm, match);
    const expanded = match || relays.some((relay) => relay.visible);
    return { ...city, relays: relays, visible: expanded };
  });
}

function searchRelays(
  relays: Array<IRelayLocationRelayRedux>,
  searchTerm: string,
  cityMatch: boolean,
): Array<RelayLocationRelayWithVisibility> {
  return relays.map((relay) => ({
    ...relay,
    visible: searchTerm === '' || cityMatch || searchMatch(searchTerm, relay.hostname),
  }));
}

export function getLocationsExpandedBySearch(
  countries: Array<IRelayLocationCountryRedux>,
  searchTerm: string,
): Array<RelayLocation> {
  return countries.reduce((locations, country) => {
    const cityLocations = getCityLocationsExpandecBySearch(
      country.cities,
      country.code,
      searchTerm,
    );
    const cityMatches = country.cities.some(
      (city) => searchMatch(searchTerm, city.code) || searchMatch(searchTerm, city.name),
    );
    const location = { country: country.code };
    const expanded = cityMatches || cityLocations.length > 0;
    return expanded ? [...locations, ...cityLocations, location] : locations;
  }, [] as Array<RelayLocation>);
}

function getCityLocationsExpandecBySearch(
  cities: Array<IRelayLocationCityRedux>,
  countryCode: string,
  searchTerm: string,
): Array<RelayLocation> {
  return cities.reduce((locations, city) => {
    const expanded =
      city.relays.filter((relay) => searchMatch(searchTerm, relay.hostname)).length > 0;
    const location: RelayLocation = { country: countryCode, city: city.code };
    return expanded ? [...locations, location] : locations;
  }, [] as Array<RelayLocation>);
}

export function searchMatch(searchTerm: string, value: string): boolean {
  return value.toLowerCase().includes(searchTerm.toLowerCase());
}

export function filterSpecialLocations<T>(
  searchTerm: string,
  locations: Array<SpecialLocation<T>>,
): Array<SpecialLocation<T>> {
  return locations.filter((location) => searchMatch(searchTerm, location.label));
}
