import type { CountryLocation } from '../types';
import { searchCityAndRelays } from './search-city-and-relays';
import { searchMatchesLocation } from './search-matches-location';

export function searchCountryAndCities(
  country: CountryLocation,
  searchTerm: string,
): CountryLocation | undefined {
  const citiesResult = country.cities
    .map((city) => searchCityAndRelays(city, searchTerm))
    .filter((city) => city !== undefined);
  if (citiesResult.length > 0) {
    return { ...country, expanded: true, cities: citiesResult };
  }
  if (searchMatchesLocation(country.label, searchTerm)) {
    return country;
  }
  return undefined;
}
