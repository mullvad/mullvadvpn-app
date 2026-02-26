import { ObfuscationType } from '../../../../../shared/daemon-rpc-types';
import { useObfuscation } from '../../../../features/anti-censorship/hooks';
import { useMultihop } from '../../../../features/multihop/hooks';
import { lwoFilterActive } from '../../../../lib/filter-locations';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useLwoFilterActive() {
  const { obfuscation } = useObfuscation();
  const { locationType } = useSelectLocationViewContext();
  const { multihop } = useMultihop();

  return lwoFilterActive(obfuscation === ObfuscationType.lwo, locationType, multihop);
}
