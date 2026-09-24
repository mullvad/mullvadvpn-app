import { ObfuscationType } from '../../../../shared/daemon-rpc-types';
import { useObfuscation } from '../../anti-censorship/hooks';
import { useMultihop } from '../../multihop/hooks';
import { LocationType } from '../types';
import { isLwoFilterActive } from '../utils/is-lwo-filter-active';

export function useIsLwoFilterActive(locationType: LocationType) {
  const { obfuscation } = useObfuscation();
  const { multihop } = useMultihop();

  if (locationType === LocationType.entryAutomatic) {
    return false;
  }

  return isLwoFilterActive(obfuscation === ObfuscationType.lwo, locationType, multihop);
}
