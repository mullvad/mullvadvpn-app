import { LocationType } from '../types';
import { isOwnershipFilterActive } from '../utils';
import { useOwnership } from './use-ownership';

export function useIsOwnershipFilterActive(locationType: LocationType) {
  const { entryOwnership, exitOwnership } = useOwnership();
  const activeOwnership = locationType === LocationType.entry ? entryOwnership : exitOwnership;

  return isOwnershipFilterActive(activeOwnership);
}
