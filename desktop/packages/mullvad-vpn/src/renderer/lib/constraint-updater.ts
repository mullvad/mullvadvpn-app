import { useCallback } from 'react';

import { getDefaultRelaySettingsNormal } from '../../main/default-settings';
import { IRelaySettingsNormal, wrapConstraint } from '../../shared/daemon-rpc-types';
import { useAppContext } from '../context';
import { NormalRelaySettingsRedux } from '../redux/settings/reducers';
import { useNormalRelaySettings } from './relay-settings-hooks';

export function wrapRelaySettingsOrDefault(
  relaySettings?: NormalRelaySettingsRedux,
): IRelaySettingsNormal {
  if (relaySettings) {
    const wgIpVersion = wrapConstraint(relaySettings.wireguard.ipVersion);
    const wgEntryLocation = wrapConstraint(relaySettings.wireguard.entryLocation);
    const location = wrapConstraint(relaySettings.location);

    return {
      providers: [...relaySettings.providers],
      ownership: relaySettings.ownership,
      wireguardConstraints: {
        ipVersion: wgIpVersion,
        useMultihop: relaySettings.wireguard.useMultihop,
        entryLocation: wgEntryLocation,
      },
      location,
    };
  }

  const defaultSettings = getDefaultRelaySettingsNormal();

  return defaultSettings;
}

type RelaySettingsUpdateFunction = (settings: IRelaySettingsNormal) => IRelaySettingsNormal;

export function useRelaySettingsModifier() {
  const relaySettings = useNormalRelaySettings();

  return useCallback(
    (fn: RelaySettingsUpdateFunction) => {
      const settings = wrapRelaySettingsOrDefault(relaySettings);
      return fn(settings);
    },
    [relaySettings],
  );
}

export function useRelaySettingsUpdater() {
  const { setRelaySettings } = useAppContext();
  const modifyRelaySettings = useRelaySettingsModifier();

  return useCallback(
    async (fn: RelaySettingsUpdateFunction) => {
      const modifiedSettings = modifyRelaySettings(fn);
      await setRelaySettings({ normal: modifiedSettings });
    },
    [setRelaySettings, modifyRelaySettings],
  );
}
