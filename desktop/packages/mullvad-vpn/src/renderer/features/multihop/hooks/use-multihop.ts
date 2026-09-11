import React from 'react';

import {
  type LiftedConstraint,
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
      multihop?: MultihopMode;
      entryLocation?: LiftedConstraint<RelayLocation>;
      exitLocation?: RelayLocation;
    }) => {
      try {
        await relaySettingsUpdater((settings) => {
          if (multihop) {
            settings.wireguardConstraints.multihop = multihop;
          }
          if (entryLocation) {
            settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
          }
          if (exitLocation) {
            settings.location = wrapConstraint(exitLocation);
          }

          return settings;
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set multihop', message);
      }
    },
    [relaySettingsUpdater],
  );

  const entryLocation = normalRelaySettings?.wireguard.entryLocation;
  const exitLocation = normalRelaySettings?.location;

  return { multihop, setMultihop, entryLocation, exitLocation };
}
