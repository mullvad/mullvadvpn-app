import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDaitaDirectOnly() {
  const daitaDirectOnly = useSelector(
    (state) => state.settings.wireguard.daita?.directOnly ?? false,
  );
  const { setDaitaDirectOnly: contextSetDaitaDirectOnly } = useAppContext();

  const setDaitaDirectOnly = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetDaitaDirectOnly(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set daita direct only', message);
      }
    },
    [contextSetDaitaDirectOnly],
  );

  return { daitaDirectOnly, setDaitaDirectOnly };
}
