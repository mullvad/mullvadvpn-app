import { useCallback } from 'react';

import { PersonalVpnConfig, SetPersonalVpnConfigError } from '../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../context';
import { useSettingsPersonalVpn } from '../../../redux/hooks';

export function usePersonalVpn() {
  const { setPersonalVpnConfig, setPersonalVpnEnabled, clearPersonalVpn } = useAppContext();
  const { personalVpnConfig, personalVpnEnabled, personalVpnStats } = useSettingsPersonalVpn();

  const save = useCallback(
    (config: PersonalVpnConfig): Promise<SetPersonalVpnConfigError> => setPersonalVpnConfig(config),
    [setPersonalVpnConfig],
  );

  const setEnabled = useCallback(
    (enabled: boolean) => setPersonalVpnEnabled(enabled),
    [setPersonalVpnEnabled],
  );

  const clear = useCallback(async (): Promise<SetPersonalVpnConfigError> => {
    await setPersonalVpnEnabled(false);
    return clearPersonalVpn();
  }, [clearPersonalVpn, setPersonalVpnEnabled]);

  return {
    config: personalVpnConfig,
    enabled: personalVpnEnabled,
    stats: personalVpnStats,
    save,
    setEnabled,
    clear,
  };
}
