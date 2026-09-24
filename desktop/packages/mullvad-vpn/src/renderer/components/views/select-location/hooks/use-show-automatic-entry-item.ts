import { useConnection } from '../../../../features/tunnel/hooks';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useEntryType } from './use-entry-type';

export function useShowAutomaticEntryItem() {
  const {
    status: { state },
  } = useConnection();
  const { isolatedItem } = useSelectLocationViewContext();
  const entryType = useEntryType();

  const isConnectedOrConnecting = state === 'connected' || state === 'connecting';

  if (
    isConnectedOrConnecting &&
    entryType === 'entryAutomatic' &&
    // Automatic entry can never be isolated
    isolatedItem === undefined
  ) {
    return true;
  }

  return false;
}
