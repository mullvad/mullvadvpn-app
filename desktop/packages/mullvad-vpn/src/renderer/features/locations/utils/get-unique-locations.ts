import type { RecentEntryLocation, RecentExitLocation } from '../types';

export function getUniqueLocations<T extends RecentEntryLocation | RecentExitLocation>(
  locations: T[],
  limit: number,
): T[] {
  return [...new Set(locations)].slice(0, limit);
}
