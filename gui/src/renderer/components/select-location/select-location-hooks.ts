import { useCallback } from 'react';

import BridgeSettingsBuilder from '../../../shared/bridge-settings-builder';
import { RelaySettingsUpdate } from '../../../shared/daemon-rpc-types';
import log from '../../../shared/logging';
import RelaySettingsBuilder from '../../../shared/relay-settings-builder';
import { useAppContext } from '../../context';
import { createWireguardRelayUpdater } from '../../lib/constraint-updater';
import { useHistory } from '../../lib/history';
import { useSelector } from '../../redux/store';
import {
  LocationSelection,
  LocationSelectionType,
  LocationType,
  SpecialBridgeLocationType,
} from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';

export function useOnSelectLocation() {
  const history = useHistory();
  const { updateRelaySettings } = useAppContext();
  const { locationType } = useSelectLocationContext();
  const baseRelaySettings = useSelector((state) => state.settings.relaySettings);

  const onSelectLocation = useCallback(
    async (relayUpdate: RelaySettingsUpdate) => {
      // dismiss the view first
      history.dismiss();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the exit location: ${error.message}`);
      }
    },
    [history],
  );

  const onSelectExitLocation = useCallback(
    async (relayLocation: LocationSelection<never>) => {
      const relayUpdate = RelaySettingsBuilder.normal()
        .location.fromRaw(relayLocation.value)
        .build();
      await onSelectLocation(relayUpdate);
    },
    [onSelectLocation],
  );
  const onSelectEntryLocation = useCallback(
    async (entryLocation: LocationSelection<never>) => {
      const relayUpdate = createWireguardRelayUpdater(baseRelaySettings)
        .tunnel.wireguard((wireguard) => wireguard.entryLocation.exact(entryLocation.value))
        .build();
      await onSelectLocation(relayUpdate);
    },
    [onSelectLocation],
  );

  return locationType === LocationType.exit ? onSelectExitLocation : onSelectEntryLocation;
}

export function useOnSelectBridgeLocation() {
  const history = useHistory();
  const { updateBridgeSettings } = useAppContext();

  return useCallback(
    async (location: LocationSelection<SpecialBridgeLocationType>) => {
      // dismiss the view first
      history.dismiss();

      let bridgeUpdate;
      if (location.type === LocationSelectionType.relay) {
        bridgeUpdate = new BridgeSettingsBuilder().location.fromRaw(location.value).build();
      } else if (
        location.type === LocationSelectionType.special &&
        location.value === SpecialBridgeLocationType.closestToExit
      ) {
        bridgeUpdate = new BridgeSettingsBuilder().location.any().build();
      }

      if (bridgeUpdate) {
        try {
          await updateBridgeSettings(bridgeUpdate);
        } catch (e) {
          const error = e as Error;
          log.error(`Failed to select the bridge location: ${error.message}`);
        }
      }
    },
    [history, updateBridgeSettings],
  );
}
