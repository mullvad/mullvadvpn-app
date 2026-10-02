import React from 'react';

import {
  type LiftedConstraint,
  type RelayLocation,
  wrapConstraint,
} from '../../../../shared/daemon-rpc-types';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useSelector } from '../../../redux/store';

export function useRelayLocations() {
  const relayLocations = useSelector((state) => state.settings.relayLocations);
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const selectEntryRelayLocation = React.useCallback(
    async (entryLocation: LiftedConstraint<RelayLocation>) => {
      await relaySettingsUpdater((settings) => {
        settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  const selectExitRelayLocation = React.useCallback(
    async (relayLocation: LiftedConstraint<RelayLocation>) => {
      await relaySettingsUpdater((settings) => {
        settings.location = wrapConstraint(relayLocation);
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  return {
    relayLocations,
    selectEntryRelayLocation,
    selectExitRelayLocation,
  };
}
