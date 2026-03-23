import type { CityLocation } from '../types';
import { searchMatchesLocation } from './search-matches-location';

export function searchCityAndRelays(
  city: CityLocation,
  searchTerm: string,
): CityLocation | undefined {
  const relaysResult = city.relays.filter((relay) =>
    searchMatchesLocation(relay.label, searchTerm),
  );
  if (relaysResult.length > 0) {
    return { ...city, expanded: true, relays: relaysResult };
  }
  if (searchMatchesLocation(city.label, searchTerm)) {
    return city;
  }

  return undefined;
}
