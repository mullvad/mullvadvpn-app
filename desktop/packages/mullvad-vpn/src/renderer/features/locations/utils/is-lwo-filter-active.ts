import { LocationType } from '../types';

export function isLwoFilterActive(lwo: boolean, locationType: LocationType, multihop: boolean) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;

  return lwo && isEntry;
}
