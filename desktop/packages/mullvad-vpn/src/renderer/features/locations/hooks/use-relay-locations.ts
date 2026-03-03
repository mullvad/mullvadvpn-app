import React from 'react';

import {
  type RelayLocation,
  type RelaySettings,
  wrapConstraint,
} from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useRelaySettingsModifier } from '../../../lib/constraint-updater';
import { useSelector } from '../../../redux/store';

export function useRelayLocations() {
  const relayLocations = useSelector((state) => state.settings.relayLocations);
  const { setRelaySettings } = useAppContext();
  const relaySettingsModifier = useRelaySettingsModifier();

  const selectLocation = React.useCallback(
    async (relaySettings: RelaySettings) => {
      try {
        await setRelaySettings(relaySettings);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the location: ${error.message}`);
      }
    },
    [setRelaySettings],
  );

  const selectEntryRelayLocation = React.useCallback(
    async (entryLocation: RelayLocation) => {
      const settings = relaySettingsModifier((settings) => {
        settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
        return settings;
      });
      await selectLocation({ normal: settings });
    },
    [relaySettingsModifier, selectLocation],
  );

  const selectExitRelayLocation = React.useCallback(
    async (relayLocation: RelayLocation) => {
      const settings = relaySettingsModifier((settings) => ({
        ...settings,
        location: wrapConstraint(relayLocation),
      }));
      await selectLocation({ normal: settings });
    },
    [relaySettingsModifier, selectLocation],
  );

  return {
    relayLocations,
    selectEntryRelayLocation,
    selectExitRelayLocation,
  };
}
