import type { CountryLocation, GeographicalLocation } from '../select-location-types';

export function createLocationMap(locations: CountryLocation[]): Map<string, GeographicalLocation> {
  const map = new Map<string, GeographicalLocation>();
  locations.forEach((location) => {
    if (location.type === 'country') {
      map.set(location.details.country, location);
      location.cities.forEach((city) => {
        map.set(city.details.city, city);
        city.relays.forEach((relay) => {
          map.set(relay.details.hostname, relay);
        });
      });
    }
  });
  return map;
}
