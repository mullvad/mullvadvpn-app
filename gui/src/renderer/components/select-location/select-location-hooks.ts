import { useCallback } from 'react';

import BridgeSettingsBuilder from '../../../shared/bridge-settings-builder';
import {
  BridgeSettings,
  RelayLocation,
  RelaySettings,
  wrapConstraint,
} from '../../../shared/daemon-rpc-types';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { useRelaySettingsModifier } from '../../lib/constraint-updater';
import { useHistory } from '../../lib/history';
import { LocationType, SpecialBridgeLocationType } from './select-location-types';
import { useSelectLocationContext } from './SelectLocationContainer';

export function useOnSelectExitLocation() {
  const onSelectLocation = useOnSelectLocation();
  const history = useHistory();
  const relaySettingsModifier = useRelaySettingsModifier();
  const { connectTunnel } = useAppContext();

  const onSelectRelay = useCallback(
    async (relayLocation: RelayLocation) => {
      const settings = relaySettingsModifier((settings) => ({
        ...settings,
        location: wrapConstraint(relayLocation),
      }));
      history.pop();
      await onSelectLocation({ normal: settings });
      await connectTunnel();
    },
    [history, relaySettingsModifier],
  );

  const onSelectSpecial = useCallback((_location: undefined) => {
    throw new Error('relayLocation should never be undefined');
  }, []);

  return [onSelectRelay, onSelectSpecial] as const;
}

export function useOnSelectEntryLocation() {
  const onSelectLocation = useOnSelectLocation();
  const { setLocationType } = useSelectLocationContext();
  const relaySettingsModifier = useRelaySettingsModifier();

  const onSelectRelay = useCallback(
    async (entryLocation: RelayLocation) => {
      setLocationType(LocationType.exit);
      const settings = relaySettingsModifier((settings) => {
        settings.wireguardConstraints.entryLocation = wrapConstraint(entryLocation);
        return settings;
      });
      await onSelectLocation({ normal: settings });
    },
    [relaySettingsModifier],
  );

  const onSelectSpecial = useCallback(
    async (_location: 'any') => {
      setLocationType(LocationType.exit);
      const settings = relaySettingsModifier((settings) => {
        settings.wireguardConstraints.entryLocation = 'any';
        return settings;
      });
      await onSelectLocation({ normal: settings });
    },
    [relaySettingsModifier],
  );

  return [onSelectRelay, onSelectSpecial] as const;
}

function useOnSelectLocation() {
  const { updateRelaySettings } = useAppContext();

  return useCallback(async (relaySettings: RelaySettings) => {
    try {
      await updateRelaySettings(relaySettings);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to select the location: ${error.message}`);
    }
  }, []);
}

export function useOnSelectBridgeLocation() {
  const { updateBridgeSettings } = useAppContext();
  const { setLocationType } = useSelectLocationContext();

  const setLocation = useCallback(async (bridgeUpdate: BridgeSettings) => {
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

  const onSelectRelay = useCallback((location: RelayLocation) => {
    const bridgeUpdate = new BridgeSettingsBuilder().location.fromRaw(location).build();
    return setLocation(bridgeUpdate);
  }, []);

  const onSelectSpecial = useCallback((location: SpecialBridgeLocationType) => {
    switch (location) {
      case SpecialBridgeLocationType.closestToExit: {
        const bridgeUpdate = new BridgeSettingsBuilder().location.any().build();
        return setLocation(bridgeUpdate);
      }
    }
  }, []);

  return [onSelectRelay, onSelectSpecial] as const;
}
