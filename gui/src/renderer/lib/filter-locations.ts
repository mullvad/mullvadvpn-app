import { Ownership, RelayEndpointType, RelayLocation } from '../../shared/daemon-rpc-types';
import { relayLocations } from '../../shared/gettext';
import { SpecialLocation } from '../components/select-location/select-location-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
  NormalRelaySettingsRedux,
} from '../redux/settings/reducers';

export enum EndpointType {
  any,
  entry,
  exit,
}

export function filterLocationsByEndPointType(
  locations: IRelayLocationCountryRedux[],
  endpointType: EndpointType,
  relaySettings?: NormalRelaySettingsRedux,
): IRelayLocationCountryRedux[] {
  return filterLocationsImpl(locations, getTunnelProtocolFilter(endpointType, relaySettings));
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
): Array<IRelayLocationCountryRedux> {
  if (searchTerm === '') {
    return countries;
  }

  return countries.reduce((countries, country) => {
    const matchingCities = searchCities(country.cities, searchTerm);
    const expanded = matchingCities.length > 0;
    const match =
      searchMatch(searchTerm, country.code) ||
      searchMatch(searchTerm, relayLocations.gettext(country.name));
    const resultingCities = match ? country.cities : matchingCities;
    return expanded || match ? [...countries, { ...country, cities: resultingCities }] : countries;
  }, [] as Array<IRelayLocationCountryRedux>);
}

function searchCities(
  cities: Array<IRelayLocationCityRedux>,
  searchTerm: string,
): Array<IRelayLocationCityRedux> {
  return cities.reduce((cities, city) => {
    const matchingRelays = city.relays.filter((relay) => searchMatch(searchTerm, relay.hostname));
    const expanded = matchingRelays.length > 0;
    const match =
      searchMatch(searchTerm, city.code) ||
      searchMatch(searchTerm, relayLocations.gettext(city.name));
    const resultingRelays = match ? city.relays : matchingRelays;
    return expanded || match ? [...cities, { ...city, relays: resultingRelays }] : cities;
  }, [] as Array<IRelayLocationCityRedux>);
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
