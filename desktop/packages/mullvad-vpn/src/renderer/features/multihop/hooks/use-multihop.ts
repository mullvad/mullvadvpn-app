import React from 'react';

import {
  MultihopMode,
  type RelayLocation,
  wrapConstraint,
} from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

export function useMultihop() {
  const normalRelaySettings = useNormalRelaySettings();
  const multihop = normalRelaySettings?.wireguard.multihop ?? 'when-needed';
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const setMultihop = React.useCallback(
    async ({
      multihop,
      entryLocation,
      exitLocation,
    }: {
      multihop: MultihopMode;
      entryLocation?: RelayLocation;
      exitLocation?: RelayLocation;
    }) => {
      try {
        await relaySettingsUpdater((settings) => {
          if (entryLocation) {
            settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
          }
          if (exitLocation) {
            settings.location = wrapConstraint(exitLocation);
          }
          settings.wireguardConstraints.multihop = multihop;
          return settings;
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set multihop', message);
      }
    },
    [relaySettingsUpdater],
  );

  return { multihop, setMultihop };
}
