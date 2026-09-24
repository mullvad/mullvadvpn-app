import { ObfuscationType } from '../../../../shared/daemon-rpc-types';
import { useObfuscation } from '../../anti-censorship/hooks';
import { useMultihop } from '../../multihop/hooks';
import { LocationType } from '../types';
import { isQuicFilterActive } from '../utils/is-quic-filter-active';

export function useIsQuicFilterActive(locationType: LocationType) {
  const { obfuscation } = useObfuscation();
  const { multihop } = useMultihop();

  if (locationType === LocationType.entryAutomatic) {
    return false;
  }

  return isQuicFilterActive(obfuscation === ObfuscationType.quic, locationType, multihop);
}
