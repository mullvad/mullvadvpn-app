import { useConnection } from '../../tunnel/hooks';
import { useMultihop } from './use-multihop';

export function useIsUsingMagicMultihop() {
  const { multihop } = useMultihop();
  const { entryHostname } = useConnection();

  if (multihop === 'when-needed' && entryHostname) {
    return true;
  }

  return false;
}
