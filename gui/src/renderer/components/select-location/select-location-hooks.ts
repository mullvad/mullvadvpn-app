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

export function useOnSelectExitLocation() {
  const onSelectLocation = useOnSelectLocation();
  const history = useHistory();
  const { connectTunnel } = useAppContext();

  return useCallback(
    async (relayLocation: LocationSelection<undefined>) => {
      if (relayLocation.value === undefined) {
        throw new Error('relayLocation should never be undefiend');
      }

      history.pop();
      const relayUpdate = RelaySettingsBuilder.normal()
        .location.fromRaw(relayLocation.value)
        .build();
      await onSelectLocation(relayUpdate);
      await connectTunnel();
    },
    [history],
  );
}

export function useOnSelectEntryLocation() {
  const onSelectLocation = useOnSelectLocation();
  const { setLocationType } = useSelectLocationContext();
  const baseRelaySettings = useSelector((state) => state.settings.relaySettings);

  return useCallback(async (entryLocation: LocationSelection<never>) => {
    setLocationType(LocationType.exit);
    const relayUpdate = createWireguardRelayUpdater(baseRelaySettings)
      .tunnel.wireguard((wireguard) => wireguard.entryLocation.exact(entryLocation.value))
      .build();
    await onSelectLocation(relayUpdate);
  }, []);
}

function useOnSelectLocation() {
  const { updateRelaySettings } = useAppContext();

  return useCallback(async (relayUpdate: RelaySettingsUpdate) => {
    try {
      await updateRelaySettings(relayUpdate);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to select the exit location: ${error.message}`);
    }
  }, []);
}

export function useOnSelectBridgeLocation() {
  const { updateBridgeSettings } = useAppContext();
  const { setLocationType } = useSelectLocationContext();

  return useCallback(async (location: LocationSelection<SpecialBridgeLocationType>) => {
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
      setLocationType(LocationType.exit);
      try {
        await updateBridgeSettings(bridgeUpdate);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to select the bridge location: ${error.message}`);
      }
    }
  }, []);
}
