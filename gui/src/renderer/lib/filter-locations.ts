import { Ownership, RelayEndpointType, RelayLocation } from '../../shared/daemon-rpc-types';
import { SpecialLocation } from '../components/select-location/select-location-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationRedux,
  IRelayLocationRelayRedux,
  NormalRelaySettingsRedux,
} from '../redux/settings/reducers';

export enum EndpointType {
  any,
  entry,
  exit,
}

export function filterLocationsByEndPointType(
  locations: IRelayLocationRedux[],
  endpointType: EndpointType,
  relaySettings?: NormalRelaySettingsRedux,
): IRelayLocationRedux[] {
  return filterLocationsImpl(locations, getTunnelProtocolFilter(endpointType, relaySettings));
}

export function filterLocations(
  locations: IRelayLocationRedux[],
  ownership?: Ownership,
  providers?: Array<string>,
): IRelayLocationRedux[] {
  const filters = [getOwnershipFilter(ownership), getProviderFilter(providers)];

  return filters.some((filter) => filter !== undefined)
    ? filterLocationsImpl(locations, (relay) => filters.every((filter) => filter?.(relay) ?? true))
    : locations;
}

function getTunnelProtocolFilter(
  endpointType: EndpointType,
  relaySettings?: NormalRelaySettingsRedux,
): (relay: IRelayLocationRelayRedux) => boolean {
  const tunnelProtocol = relaySettings?.tunnelProtocol ?? 'any';
  const endpointTypes: Array<RelayEndpointType> = [];
  if (endpointType !== EndpointType.exit && tunnelProtocol === 'openvpn') {
    endpointTypes.push('bridge');
  } else if (tunnelProtocol === 'any') {
    endpointTypes.push('wireguard');
    if (!relaySettings?.wireguard.useMultihop) {
      endpointTypes.push('openvpn');
    }
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
  locations: Array<IRelayLocationRedux>,
  filter: (relay: IRelayLocationRelayRedux) => boolean,
): Array<IRelayLocationRedux> {
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
  countries: Array<IRelayLocationRedux>,
  searchTerm: string,
): Array<IRelayLocationRedux> {
  if (searchTerm === '') {
    return countries;
  }

  return countries.reduce((countries, country) => {
    const matchingCities = searchCities(country.cities, searchTerm);
    const expanded = matchingCities.length > 0;
    const match = search(searchTerm, country.code) || search(searchTerm, country.name);
    const resultingCities = match ? country.cities : matchingCities;
    return expanded || match ? [...countries, { ...country, cities: resultingCities }] : countries;
  }, [] as Array<IRelayLocationRedux>);
}

function searchCities(
  cities: Array<IRelayLocationCityRedux>,
  searchTerm: string,
): Array<IRelayLocationCityRedux> {
  return cities.reduce((cities, city) => {
    const matchingRelays = city.relays.filter((relay) => search(searchTerm, relay.hostname));
    const expanded = matchingRelays.length > 0;
    const match = search(searchTerm, city.code) || search(searchTerm, city.name);
    const resultingRelays = match ? city.relays : matchingRelays;
    return expanded || match ? [...cities, { ...city, relays: resultingRelays }] : cities;
  }, [] as Array<IRelayLocationCityRedux>);
}

export function getLocationsExpandedBySearch(
  countries: Array<IRelayLocationRedux>,
  searchTerm: string,
): Array<RelayLocation> {
  return countries.reduce((locations, country) => {
    const cityLocations = getCityLocationsExpandecBySearch(
      country.cities,
      country.code,
      searchTerm,
    );
    const cityMatches = country.cities.some(
      (city) => search(searchTerm, city.code) || search(searchTerm, city.name),
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
    const expanded = city.relays.filter((relay) => search(searchTerm, relay.hostname)).length > 0;
    const location: RelayLocation = { city: [countryCode, city.code] };
    return expanded ? [...locations, location] : locations;
  }, [] as Array<RelayLocation>);
}

function search(searchTerm: string, value: string): boolean {
  return value.toLowerCase().includes(searchTerm.toLowerCase());
}

export function filterSpecialLocations<T>(
  searchTerm: string,
  locations: Array<SpecialLocation<T>>,
): Array<SpecialLocation<T>> {
  return locations.filter((location) => search(searchTerm, location.label));
}
