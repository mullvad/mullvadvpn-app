import { useCallback } from 'react';

import { IDnsOptions } from '../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useDns(setting: keyof IDnsOptions['defaultOptions']) {
  const dns = useSelector((state) => state.settings.dns);
  const { setDnsOptions } = useAppContext();

  const updateBlockSetting = useCallback(
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

  return [dns, updateBlockSetting] as const;
}
