import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useEnableIpv6() {
  const enableIpv6 = useSelector((state) => state.settings.enableIpv6);
  const { setEnableIpv6: contextSetEnableIpv6 } = useAppContext();

  const setEnableIpv6 = React.useCallback(
    async (value: boolean) => {
      try {
        await contextSetEnableIpv6(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set enable IPv6', message);
      }
    },
    [contextSetEnableIpv6],
  );

  return { enableIpv6, setEnableIpv6 };
}
