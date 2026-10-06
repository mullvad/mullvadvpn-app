import { MultihopMode } from '../../../../shared/daemon-rpc-types';
import { LocationType } from '../types';

export function isQuicFilterActive(
  quic: boolean,
  locationType: LocationType,
  multihop: MultihopMode,
) {
  const isEntry = multihop !== 'never' ? locationType === 'entry' : locationType === 'exit';

  return quic && isEntry;
}
