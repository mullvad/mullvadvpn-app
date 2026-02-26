import { ObfuscationType } from '../../../../../shared/daemon-rpc-types';
import { useObfuscation } from '../../../../features/anti-censorship/hooks';
import { useMultihop } from '../../../../features/multihop/hooks';
import { quicFilterActive } from '../../../../lib/filter-locations';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useQuicFilterActive() {
  const { obfuscation } = useObfuscation();
  const { locationType } = useSelectLocationViewContext();
  const { multihop } = useMultihop();

  return quicFilterActive(obfuscation === ObfuscationType.quic, locationType, multihop);
}
