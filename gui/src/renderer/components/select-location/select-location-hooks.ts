import { useCallback } from 'react';

import {
  BridgeSettings,
  RelayLocation,
  RelaySettings,
  wrapConstraint,
} from '../../../shared/daemon-rpc-types';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { useRelaySettingsModifier } from '../../lib/constraint-updater';
import { useBridgeSettingsModifier } from '../../lib/constraint-updater';
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
    [connectTunnel, history, onSelectLocation, relaySettingsModifier],
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
    [onSelectLocation, relaySettingsModifier, setLocationType],
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
    [onSelectLocation, relaySettingsModifier, setLocationType],
  );

  return [onSelectRelay, onSelectSpecial] as const;
}

function useOnSelectLocation() {
  const { setRelaySettings } = useAppContext();

  return useCallback(
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
}

export function useOnSelectBridgeLocation() {
  const { updateBridgeSettings } = useAppContext();
  const { setLocationType } = useSelectLocationContext();
  const bridgeSettingsModifier = useBridgeSettingsModifier();

  const setLocation = useCallback(
    async (bridgeUpdate: BridgeSettings) => {
      if (bridgeUpdate) {
        setLocationType(LocationType.exit);
        try {
          await updateBridgeSettings(bridgeUpdate);
        } catch (e) {
          const error = e as Error;
          log.error(`Failed to select the bridge location: ${error.message}`);
        }
      }
    },
    [setLocationType, updateBridgeSettings],
  );

  const onSelectRelay = useCallback(
    (location: RelayLocation) => {
      return setLocation(
        bridgeSettingsModifier((bridgeSettings) => {
          bridgeSettings.type = 'normal';
          bridgeSettings.normal.location = wrapConstraint(location);
          return bridgeSettings;
        }),
      );
    },
    [bridgeSettingsModifier, setLocation],
  );

  const onSelectSpecial = useCallback(
    (location: SpecialBridgeLocationType) => {
      switch (location) {
        case SpecialBridgeLocationType.closestToExit:
          return setLocation(
            bridgeSettingsModifier((bridgeSettings) => {
              bridgeSettings.type = 'normal';
              bridgeSettings.normal.location = 'any';
              return bridgeSettings;
            }),
          );
        case SpecialBridgeLocationType.custom:
          return setLocation(
            bridgeSettingsModifier((bridgeSettings) => {
              bridgeSettings.type = 'custom';
              return bridgeSettings;
            }),
          );
      }
    },
    [bridgeSettingsModifier, setLocation],
  );

  return [onSelectRelay, onSelectSpecial] as const;
}
