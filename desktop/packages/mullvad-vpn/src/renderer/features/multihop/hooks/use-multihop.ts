import React from 'react';

import {
  type LiftedConstraint,
  type RelayLocation,
  wrapConstraint,
} from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../lib/relay-settings-hooks';

type SetMultihopParams = NeverMultihopParams | WhenNeededMultihopParams | AlwaysMultihopParams;

type NeverMultihopParams = {
  multihop: 'never';
  exit?: LiftedConstraint<RelayLocation>;
};

type WhenNeededMultihopParams = {
  multihop: 'when-needed';
  exit?: LiftedConstraint<RelayLocation>;
};

type AlwaysMultihopParams = {
  multihop: 'always';
  entry?: LiftedConstraint<RelayLocation>;
  exit?: LiftedConstraint<RelayLocation>;
};

export function useMultihop() {
  const normalRelaySettings = useNormalRelaySettings();
  const multihop = normalRelaySettings?.wireguard.multihop ?? 'when-needed';
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const setMultihop = React.useCallback(
    async (params: SetMultihopParams) => {
      try {
        await relaySettingsUpdater((settings) => {
          switch (params.multihop) {
            case 'never':
              if (params.exit) {
                settings.location = wrapConstraint(params.exit);
              }
              settings.wireguardConstraints.multihop = params.multihop;
              break;
            case 'when-needed':
              if (params.exit) {
                settings.location = wrapConstraint(params.exit);
              }
              settings.wireguardConstraints.multihop = params.multihop;
              break;
            case 'always':
              if (params.entry) {
                settings.wireguardConstraints.entryLocation = wrapConstraint(params.entry);
              }
              if (params.exit) {
                settings.location = wrapConstraint(params.exit);
              }
              break;
          }
          settings.wireguardConstraints.multihop = params.multihop;
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
