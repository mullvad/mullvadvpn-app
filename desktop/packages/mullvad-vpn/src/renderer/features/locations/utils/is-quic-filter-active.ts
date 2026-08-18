import { LocationType } from '../types';

export function isQuicFilterActive(quic: boolean, locationType: LocationType, multihop: boolean) {
  const isEntry = multihop
    ? locationType === LocationType.entry
    : locationType === LocationType.exit;

  return quic && isEntry;
}
