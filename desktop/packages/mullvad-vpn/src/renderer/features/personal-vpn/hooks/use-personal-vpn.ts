import { useCallback } from 'react';

import { CustomVpnConfig, SetCustomVpnConfigError } from '../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../context';
import { useSettingsCustomVpn } from '../../../redux/hooks';

export function usePersonalVpn() {
  const { setCustomVpnConfig, setCustomVpnEnabled, clearCustomVpn } = useAppContext();
  const { customVpnConfig, customVpnEnabled, customVpnStats } = useSettingsCustomVpn();

  const save = useCallback(
    (config: CustomVpnConfig): Promise<SetCustomVpnConfigError> => setCustomVpnConfig(config),
    [setCustomVpnConfig],
  );

  const setEnabled = useCallback(
    (enabled: boolean) => setCustomVpnEnabled(enabled),
    [setCustomVpnEnabled],
  );

  const clear = useCallback(async (): Promise<SetCustomVpnConfigError> => {
    await setCustomVpnEnabled(false);
    return clearCustomVpn();
  }, [clearCustomVpn, setCustomVpnEnabled]);

  return {
    config: customVpnConfig,
    enabled: customVpnEnabled,
    stats: customVpnStats,
    save,
    setEnabled,
    clear,
  };
}
