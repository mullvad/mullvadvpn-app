import { useConnection } from '../../../../../../features/tunnel/hooks';
import { useEntryType } from '../../../hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

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
