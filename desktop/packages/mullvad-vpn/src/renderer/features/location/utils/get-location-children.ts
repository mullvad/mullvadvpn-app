import type { AnyLocation, GeographicalLocation } from '../types';

export function getLocationChildren(location: AnyLocation): GeographicalLocation[] {
  if (location.type === 'customList') {
    return location.locations;
  } else if (location.type === 'country') {
    return location.cities;
  } else if (location.type === 'city') {
    return location.relays;
  } else {
    return [];
  }
}
