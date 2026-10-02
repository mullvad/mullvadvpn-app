import { useMultihop } from '../../../../features/multihop/hooks';
import { useConnection } from '../../../../features/tunnel/hooks';

export function useEntryType() {
  const { multihop } = useMultihop();
  const { entryHostname } = useConnection();

  if (multihop === 'always') {
    return 'entry';
  }

  if (multihop === 'when-needed' && entryHostname) {
    return 'entryAutomatic';
  }

  return undefined;
}
