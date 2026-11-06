import { useIsUsingMagicMultihop, useMultihop } from '../../../../features/multihop/hooks';

export function useEntryType() {
  const { multihop } = useMultihop();
  const isUsingMagicMultihop = useIsUsingMagicMultihop();

  if (multihop === 'always') {
    return 'entry';
  }

  if (isUsingMagicMultihop) {
    return 'entryAutomatic';
  }

  return undefined;
}
