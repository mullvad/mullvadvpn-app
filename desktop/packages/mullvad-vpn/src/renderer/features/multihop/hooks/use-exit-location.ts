import React from 'react';

import { type RelayLocation, wrapConstraint } from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

export function useExitLocation() {
  const normalRelaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const setExitLocation = React.useCallback(
    async (exitLocation: RelayLocation) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.location = wrapConstraint(exitLocation);
          return settings;
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set exit location', message);
      }
    },
    [relaySettingsUpdater],
  );

  const exitLocation = normalRelaySettings?.location;

  return { exitLocation, setExitLocation };
}
