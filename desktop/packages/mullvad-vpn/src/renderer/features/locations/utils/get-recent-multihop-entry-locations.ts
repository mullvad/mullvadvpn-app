import type { AnyLocation, RecentLocation } from '../types';

export const getRecentMultihopEntryLocations = (
  recentLocations?: RecentLocation[],
): AnyLocation[] | undefined => {
  if (!recentLocations) {
    return undefined;
  }

  const addedLocations = new Set();
  const multihopLocations = recentLocations
    .filter((location) => location.type === 'multihop')
    .map((location) => location.entry)
    .filter((location) => {
      if (addedLocations.has(location)) {
        return false;
      }
      addedLocations.add(location);
      return true;
    })
    .slice(0, 3);

  return multihopLocations.length > 0 ? multihopLocations : undefined;
};
