import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDaitaEnabled() {
  const daitaEnabled = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const { setEnableDaita: contextSetEnableDaita } = useAppContext();

  const setDaitaEnabled = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetEnableDaita(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set enable daita', message);
      }
    },
    [contextSetEnableDaita],
  );

  return { daitaEnabled, setDaitaEnabled };
}
