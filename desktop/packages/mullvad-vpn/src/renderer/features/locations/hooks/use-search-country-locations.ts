import React from 'react';

import type { CityLocation, CountryLocation, RelayLocation } from '../types';
import { searchMatchesLocation } from '../utils';

export function useSearchCountryLocations(
  locations: CountryLocation[],
  searchTerm: string,
): CountryLocation[] {
  return React.useMemo(() => formatCountriesResult(locations, searchTerm), [locations, searchTerm]);
}

export function formatCountriesResult(countries: CountryLocation[], searchTerm: string) {
  if (!searchTerm) {
    return countries;
  }
  return countries
    .map((country) => {
      const citiesResult = formatCitiesResult(country, searchTerm);
      if (citiesResult.length > 0) {
        return { ...country, expanded: true, cities: citiesResult };
      }
      if (searchMatchesLocation(country.label, searchTerm)) {
        return country;
      }
      return undefined;
    })
    .filter((country) => country !== undefined);
}

export function formatCitiesResult(country: CountryLocation, searchTerm: string): CityLocation[] {
  return country.cities
    .map((city) => {
      const relaysResult = formatRelaysResult(city, searchTerm);
      if (relaysResult.length > 0) {
        return { ...city, expanded: true, relays: relaysResult };
      }
      if (searchMatchesLocation(city.label, searchTerm)) {
        return city;
      }

      return undefined;
    })
    .filter((city) => city !== undefined);
}

export function formatRelaysResult(city: CityLocation, searchTerm: string): RelayLocation[] {
  return city.relays.filter((relay) => searchMatchesLocation(relay.label, searchTerm));
}
