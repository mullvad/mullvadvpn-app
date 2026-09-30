import { ObfuscationType } from '../../../../shared/daemon-rpc-types';
import { useObfuscation } from '../../anti-censorship/hooks';
import { useMultihop } from '../../multihop/hooks';
import type { LocationType } from '../types';
import { isQuicFilterActive } from '../utils/is-quic-filter-active';

export function useIsQuicFilterActive(locationType: LocationType) {
  const { obfuscation } = useObfuscation();
  const { multihop } = useMultihop();

  return isQuicFilterActive(obfuscation === ObfuscationType.quic, locationType, multihop);
}
