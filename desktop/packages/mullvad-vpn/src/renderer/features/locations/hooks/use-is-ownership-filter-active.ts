import { LocationType } from '../types';
import { isOwnershipFilterActive } from '../utils';
import { useOwnership } from './use-ownership';

export function useIsOwnershipFilterActive(locationType: LocationType) {
  const { ownership } = useOwnership(locationType);

  if (locationType === LocationType.entryAutomatic) {
    return false;
  }

  return isOwnershipFilterActive(ownership);
}
