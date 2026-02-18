import type { CityLocation, CountryLocation, RelayLocation } from '../select-location-types';
import { searchMatchesLocation } from '../utils';

export function useSearchLocations(
  locations: CountryLocation[],
  searchTerm: string,
): CountryLocation[] {
  if (!searchTerm) {
    return locations;
  }
  const result: CountryLocation[] = [];
  locations.forEach((country) => {
    const countriesResult: CountryLocation[] = [];
    const citiesResult: CityLocation[] = [];
    const relaysResult: RelayLocation[] = [];
    const pushedCities = new Set<string>();
    const pushedCountries = new Set<string>();

    const countryKey = country.details.country;
    country.cities.forEach((city, index) => {
      const cityKey = `${city.details.country}-${city.details.city}`;
      city.relays.forEach((relay, index) => {
        if (searchMatchesLocation(relay.searchText, searchTerm)) {
          relaysResult.push(relay);
        }
        // If it's the last relay in the city and we have a match, push the city with
        // a copy of relay result array. Then reset the relay result array for the next city
        if (index === city.relays.length - 1 && relaysResult.length > 0) {
          citiesResult.push({ ...city, expanded: true, relays: Array.from(relaysResult) });
          pushedCities.add(cityKey);
          relaysResult.length = 0;
        }
      });
      if (!pushedCities.has(cityKey)) {
        if (searchMatchesLocation(city.searchText, searchTerm)) {
          citiesResult.push(city);
        }
      }
      // Handle countries in the same way as was described for cities above
      if (index === country.cities.length - 1 && citiesResult.length > 0) {
        countriesResult.push({ ...country, expanded: true, cities: Array.from(citiesResult) });
        pushedCountries.add(countryKey);
        citiesResult.length = 0;
      }
    });
    // If country not already has been pushed and matches search, add country with all locations
    if (!pushedCountries.has(countryKey)) {
      if (searchMatchesLocation(country.searchText, searchTerm)) {
        countriesResult.push(country);
      }
    }
    result.push(...countriesResult);
  });

  return result;
}
