import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAllowLan() {
  const allowLan = useSelector((state) => state.settings.allowLan);
  const { setAllowLan: contextSetAllowLan } = useAppContext();

  const setAllowLan = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetAllowLan(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set allow lan', message);
      }
    },
    [contextSetAllowLan],
  );

  return { allowLan, setAllowLan };
}
