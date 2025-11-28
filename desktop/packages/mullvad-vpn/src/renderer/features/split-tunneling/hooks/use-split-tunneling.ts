import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useSplitTunneling() {
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);
  const { setSplitTunnelingState: contextSetSplitTunnelingState } = useAppContext();

  const setSplitTunnelingState = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetSplitTunnelingState(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set split tunneling state', message);
      }
    },
    [contextSetSplitTunnelingState],
  );

  return { splitTunnelingEnabled, setSplitTunnelingState };
}
