import { MultihopMode } from '../../../../shared/daemon-rpc-types';
import { LocationType } from '../types';

export function isDaitaFilterActive(
  daita: boolean,
  locationType: LocationType,
  multihop: MultihopMode,
) {
  const isEntry = multihop !== 'never' ? locationType === 'entry' : locationType === 'exit';

  return daita && isEntry;
}
