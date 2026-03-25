import type { AnyLocation, RecentLocation } from '../types';

export const getRecentSinglehopLocations = (
  recentLocations?: RecentLocation[],
): AnyLocation[] | undefined => {
  if (!recentLocations) {
    return undefined;
  }

  const addedLocations = new Set();
  const singlehopLocations = recentLocations
    .filter((location) => location.type === 'singlehop')
    .filter((location) => {
      if (addedLocations.has(location.location)) {
        return false;
      }
      addedLocations.add(location.location);
      return true;
    })
    .map((location) => location.location)
    .slice(0, 3);

  return singlehopLocations.length > 0 ? singlehopLocations : undefined;
};
