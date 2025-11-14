import { useCallback } from 'react';

import { IDnsOptions } from '../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useSetDnsOption(setting: keyof IDnsOptions['defaultOptions']) {
  const dns = useSelector((state) => state.settings.dns);
  const { setDnsOptions } = useAppContext();

  const updateDnsOption = useCallback(
    (enabled: boolean) =>
      setDnsOptions({
        ...dns,
        defaultOptions: {
          ...dns.defaultOptions,
          [setting]: enabled,
        },
      }),
    [setting, dns, setDnsOptions],
  );

  return updateDnsOption;
}
