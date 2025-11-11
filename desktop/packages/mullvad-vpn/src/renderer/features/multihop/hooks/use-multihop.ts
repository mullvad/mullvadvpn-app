import React from 'react';

import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

export function useMultihop() {
  const normalRelaySettings = useNormalRelaySettings();
  const multihop = normalRelaySettings?.wireguard.useMultihop ?? false;
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const setMultihop = React.useCallback(
    async (enabled: boolean) => {
      await relaySettingsUpdater((settings) => {
        settings.wireguardConstraints.useMultihop = enabled;
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  return { multihop, setMultihop };
}
