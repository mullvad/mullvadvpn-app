import type { CustomListLocation } from '../types';
import { searchCityAndRelays } from './search-city-and-relays';
import { searchCountryAndCities } from './search-country-and-cities';
import { searchMatchesLocation } from './search-matches-location';

export function searchCustomListAndLocations(
  customList: CustomListLocation,
  searchTerm: string,
): CustomListLocation | undefined {
  const locationsResult = customList.locations.filter((location) => {
    if (location.type === 'relay') {
      return searchMatchesLocation(location.label, searchTerm);
    } else if (location.type === 'city') {
      const cityResult = searchCityAndRelays(location, searchTerm);
      return cityResult !== undefined;
    } else if (location.type === 'country') {
      const countryResult = searchCountryAndCities(location, searchTerm);
      return countryResult !== undefined;
    }
    return false;
  });

  if (locationsResult.length > 0) {
    return { ...customList, expanded: true, locations: locationsResult };
  }

  const customListMatchesSearch = searchMatchesLocation(customList.label, searchTerm);
  if (customListMatchesSearch) {
    return customList;
  }
  return undefined;
}
