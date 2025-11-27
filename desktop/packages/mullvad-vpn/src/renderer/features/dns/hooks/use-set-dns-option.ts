import { useCallback } from 'react';

import { IDnsOptions } from '../../../../shared/daemon-rpc-types';
import { useSelector } from '../../../redux/store';
import { useDns } from './use-dns';

export function useSetDnsOption(setting: keyof IDnsOptions['defaultOptions']) {
  const dns = useSelector((state) => state.settings.dns);
  const { setDns } = useDns();

  const updateDnsOption = useCallback(
    (enabled: boolean) =>
      setDns({
        ...dns,
        defaultOptions: {
          ...dns.defaultOptions,
          [setting]: enabled,
        },
      }),
    [setDns, dns, setting],
  );

  return updateDnsOption;
}
