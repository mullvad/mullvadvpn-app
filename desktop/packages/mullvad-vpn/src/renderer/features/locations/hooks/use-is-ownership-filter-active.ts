import { Ownership } from '../../../../shared/daemon-rpc-types';
import { LocationType } from '../types';
import { useOwnership } from './use-ownership';

export function useIsOwnershipFilterActive(locationType: LocationType) {
  const { entryOwnership, exitOwnership } = useOwnership();
  const activeOwnership = locationType === LocationType.entry ? entryOwnership : exitOwnership;

  const isOwnershipFilterActive = activeOwnership !== Ownership.any;

  return isOwnershipFilterActive;
}
