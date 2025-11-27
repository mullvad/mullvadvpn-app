import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useLockdownMode() {
  const lockdownMode = useSelector((state) => state.settings.lockdownMode);
  const { setLockdownMode: contextSetLockdownMode } = useAppContext();

  const setLockdownMode = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetLockdownMode(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set lockdown mode', message);
      }
    },
    [contextSetLockdownMode],
  );

  return { lockdownMode, setLockdownMode };
}
