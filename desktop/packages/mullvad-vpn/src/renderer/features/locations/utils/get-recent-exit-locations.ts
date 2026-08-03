import type { AnyLocation, RecentLocations } from '../types';
import { getUniqueLocations } from './get-unique-locations';

export const getRecentExitLocations = (
  recentLocations?: RecentLocations,
): AnyLocation[] | undefined => {
  if (!recentLocations) {
    return undefined;
  }

  const { exits } = recentLocations;
  const uniqueExitLocations = getUniqueLocations(exits);

  return uniqueExitLocations.length > 0 ? uniqueExitLocations : undefined;
};
