import React from 'react';

import { IDnsOptions } from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDns() {
  const dns = useSelector((state) => state.settings.dns);
  const { setDnsOptions: contextSetDns } = useAppContext();

  const setDns = React.useCallback(
    async (value: IDnsOptions) => {
      try {
        await contextSetDns(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set dns', message);
      }
    },
    [contextSetDns],
  );

  return { dns, setDns };
}
