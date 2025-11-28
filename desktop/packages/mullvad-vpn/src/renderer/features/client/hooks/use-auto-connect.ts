import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAutoConnect() {
  const autoConnect = useSelector((state) => state.settings.guiSettings.autoConnect);
  const { setAutoConnect: contextSetAutoConnect } = useAppContext();

  const setAutoConnect = React.useCallback(
    (value: boolean) => {
      try {
        contextSetAutoConnect(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set auto connect', message);
      }
    },
    [contextSetAutoConnect],
  );

  return { autoConnect, setAutoConnect };
}
