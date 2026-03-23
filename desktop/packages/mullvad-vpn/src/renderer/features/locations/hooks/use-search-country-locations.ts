import React from 'react';

import type { CountryLocation } from '../types';
import { searchCountryAndCities } from '../utils';

export function useSearchCountryLocations(
  countryLocations: CountryLocation[],
  searchTerm: string,
): CountryLocation[] {
  return React.useMemo(() => {
    if (!searchTerm) {
      return countryLocations;
    }

    return countryLocations
      .map((country) => searchCountryAndCities(country, searchTerm))
      .filter((country) => country !== undefined);
  }, [countryLocations, searchTerm]);
}
