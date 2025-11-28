import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAutoStart() {
  const autoStart = useSelector((state) => state.settings.autoStart);
  const { setAutoStart: contextSetAutoStart } = useAppContext();

  const setAutoStart = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetAutoStart(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set auto start', message);
      }
    },
    [contextSetAutoStart],
  );

  return { autoStart, setAutoStart };
}
