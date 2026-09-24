import React from 'react';

import {
  type LiftedConstraint,
  type RelayLocation,
  wrapConstraint,
} from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

export function useEntryLocation() {
  const normalRelaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const setEntryLocation = React.useCallback(
    async (entryLocation: LiftedConstraint<RelayLocation>) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
          return settings;
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set entry location', message);
      }
    },
    [relaySettingsUpdater],
  );

  const entryLocation = normalRelaySettings?.wireguard.entryLocation;

  return { entryLocation, setEntryLocation };
}
