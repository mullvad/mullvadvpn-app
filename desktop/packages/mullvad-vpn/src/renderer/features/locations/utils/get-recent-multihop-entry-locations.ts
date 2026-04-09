import type { AnyLocation, RecentLocation } from '../types';
import { getUniqueLocations } from './get-unique-locations';

export const getRecentMultihopEntryLocations = (
  recentLocations?: RecentLocation[],
): AnyLocation[] | undefined => {
  if (!recentLocations) {
    return undefined;
  }

  const multihopLocations = recentLocations
    .filter((location) => location.type === 'multihop')
    .map((location) => location.entry);

  const uniqueMultihopLocations = getUniqueLocations(multihopLocations);

  return uniqueMultihopLocations.length > 0 ? uniqueMultihopLocations : undefined;
};
