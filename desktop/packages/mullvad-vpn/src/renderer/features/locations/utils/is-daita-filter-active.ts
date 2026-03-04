import { LocationType } from '../types';

export function isDaitaFilterActive(
  daita: boolean,
  directOnly: boolean,
  locationType: LocationType,
  multihop: boolean,
) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;

  return daita && (directOnly || multihop) && isEntry;
}
