import { useCallback } from 'react';

import { RelaySettings } from '../../../../../shared/daemon-rpc-types';
import log from '../../../../../shared/logging';
import { useAppContext } from '../../../../context';

export function useOnSelectLocation() {
  const { setRelaySettings } = useAppContext();

  return useCallback(
    async (relaySettings: RelaySettings) => {
      try {
        await setRelaySettings(relaySettings);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the location: ${error.message}`);
      }
    },
    [setRelaySettings],
  );
}
