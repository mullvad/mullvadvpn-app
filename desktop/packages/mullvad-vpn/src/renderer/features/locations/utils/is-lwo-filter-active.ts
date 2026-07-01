import { MultihopMode } from '../../../../shared/daemon-rpc-types';
import { LocationType } from '../types';

export function isLwoFilterActive(
  lwo: boolean,
  locationType: LocationType,
  multihop: MultihopMode,
) {
  const isEntry =
    multihop !== 'never' ? locationType === LocationType.entry : locationType === LocationType.exit;

  return lwo && isEntry;
}
