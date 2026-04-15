import type { AnyLocation } from '../types';

export function getUniqueLocations(locations: AnyLocation[], limit = 3): AnyLocation[] {
  return [...new Set(locations)].slice(0, limit);
}
