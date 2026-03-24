import type { CountryLocation, GeographicalLocation } from '../types';

export function createCountryLocationMap(
  locations: CountryLocation[],
): Map<string, GeographicalLocation> {
  const countryLocationMap = new Map<string, GeographicalLocation>();
  locations.forEach((location) => {
    if (location.type === 'country') {
      countryLocationMap.set(location.details.country, location);
      location.cities.forEach((city) => {
        countryLocationMap.set(city.details.city, city);
        city.relays.forEach((relay) => {
          countryLocationMap.set(relay.details.hostname, relay);
        });
      });
    }
  });
  return countryLocationMap;
}
