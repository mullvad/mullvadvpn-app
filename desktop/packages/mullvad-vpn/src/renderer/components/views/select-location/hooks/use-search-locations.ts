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
        if (searchMatchesLocation(relay.label, searchTerm)) {
          relaysResult.push(relay);
        }
        if (index === city.relays.length - 1 && relaysResult.length > 0) {
          citiesResult.push({ ...city, expanded: true, relays: Array.from(relaysResult) });
          pushedCities.add(cityKey);
          relaysResult.length = 0;
        }
      });
      if (!pushedCities.has(cityKey)) {
        if (searchMatchesLocation(city.label, searchTerm)) {
          citiesResult.push(city);
        }
      }
      if (index === country.cities.length - 1 && citiesResult.length > 0) {
        countriesResult.push({ ...country, expanded: true, cities: Array.from(citiesResult) });
        pushedCountries.add(countryKey);
        citiesResult.length = 0;
      }
    });
    if (!pushedCountries.has(countryKey)) {
      if (searchMatchesLocation(country.label, searchTerm)) {
        countriesResult.push(country);
      }
    }
    result.push(...countriesResult);
  });

  return result;
}
