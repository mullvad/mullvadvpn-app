import type { AnyLocation, RecentLocation } from '../types';
import { getUniqueLocations } from './get-unique-locations';

export const getRecentSinglehopLocations = (
  recentLocations?: RecentLocation[],
): AnyLocation[] | undefined => {
  if (!recentLocations) {
    return undefined;
  }

  const singlehopLocations = recentLocations
    .filter((location) => location.type === 'singlehop')
    .map((location) => location.location);

  const uniqueSinglehopLocations = getUniqueLocations(singlehopLocations);

  return uniqueSinglehopLocations.length > 0 ? uniqueSinglehopLocations : undefined;
};
