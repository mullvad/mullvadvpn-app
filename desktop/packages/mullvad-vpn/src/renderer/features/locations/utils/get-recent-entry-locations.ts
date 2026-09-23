import type { RecentEntryLocation, RecentLocations } from '../types';
import { getUniqueLocations } from './get-unique-locations';

export const getRecentEntryLocations = (
  recentLocations?: RecentLocations,
): RecentEntryLocation[] | undefined => {
  if (!recentLocations) {
    return undefined;
  }

  const { entries } = recentLocations;
  const uniqueEntryLocations = getUniqueLocations(entries);

  return uniqueEntryLocations.length > 0 ? uniqueEntryLocations : undefined;
};
