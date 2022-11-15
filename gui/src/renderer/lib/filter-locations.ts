import { Ownership, RelayLocation } from '../../shared/daemon-rpc-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationRedux,
  IRelayLocationRelayRedux,
} from '../redux/settings/reducers';

export function filterLocations(
  locations: IRelayLocationRedux[],
  providers: string[],
  ownership: Ownership,
): IRelayLocationRedux[] {
  const locationsFilteredByOwnership = filterLocationsByOwnership(locations, ownership);
  const locationsFilteredByProvider = filterLocationsByProvider(
    locationsFilteredByOwnership,
    providers,
  );

  return locationsFilteredByProvider;
}

function filterLocationsByOwnership(
  locations: IRelayLocationRedux[],
  ownership: Ownership,
): IRelayLocationRedux[] {
  if (ownership === Ownership.any) {
    return locations;
  }

  const expectOwned = ownership === Ownership.mullvadOwned;
  return filterLocationsImpl(locations, (relay) => relay.owned === expectOwned);
}

function filterLocationsByProvider(
  locations: IRelayLocationRedux[],
  providers: string[],
): IRelayLocationRedux[] {
  return providers.length === 0
    ? locations
    : filterLocationsImpl(locations, (relay) => providers.includes(relay.provider));
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
  return countries.reduce((countries, country) => {
    const matchingCities = searchCities(country.cities, searchTerm);
    const expanded = matchingCities.length > 0;
    const match = search(country.code, searchTerm) || search(country.name, searchTerm);
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
    const match = search(city.code, searchTerm) || search(city.name, searchTerm);
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
    const location = { country: country.code };
    const expanded = cityLocations.length > 0;
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
