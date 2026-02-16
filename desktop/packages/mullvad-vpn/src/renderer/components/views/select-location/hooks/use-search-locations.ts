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

    country.cities.forEach((city, index) => {
      city.relays.forEach((relay, index) => {
        if (searchMatchesLocation(relay.label, searchTerm)) {
          relaysResult.push(relay);
        }
        if (index === city.relays.length - 1 && relaysResult.length > 0) {
          citiesResult.push({ ...city, expanded: true, relays: Array.from(relaysResult) });
          pushedCities.add(`${city.details.country}-${city.details.city}`);
          relaysResult.length = 0;
        }
      });
      if (!pushedCities.has(`${city.details.country}-${city.details.city}`)) {
        if (searchMatchesLocation(city.label, searchTerm)) {
          citiesResult.push(city);
        }
      }
      if (index === country.cities.length - 1 && citiesResult.length > 0) {
        countriesResult.push({ ...country, expanded: true, cities: Array.from(citiesResult) });
        pushedCountries.add(country.details.country);
        citiesResult.length = 0;
      }
    });
    if (!pushedCountries.has(country.details.country)) {
      if (searchMatchesLocation(country.label, searchTerm)) {
        countriesResult.push(country);
      }
    }
    result.push(...countriesResult);
  });

  return result;
}
