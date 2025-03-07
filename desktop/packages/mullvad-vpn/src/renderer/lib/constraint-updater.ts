import { useCallback } from 'react';

import { getDefaultRelaySettingsNormal } from '../../main/default-settings';
import {
  BridgeSettings,
  IBridgeConstraints,
  IOpenVpnConstraints,
  IRelaySettingsNormal,
  IWireguardConstraints,
  Ownership,
  wrapConstraint,
} from '../../shared/daemon-rpc-types';
import { useAppContext } from '../context';
import {
  BridgeSettingsRedux,
  NormalBridgeSettingsRedux,
  NormalRelaySettingsRedux,
} from '../redux/settings/reducers';
import { useSelector } from '../redux/store';
import { useNormalRelaySettings } from './relay-settings-hooks';

export function wrapRelaySettingsOrDefault(
  relaySettings?: NormalRelaySettingsRedux,
): IRelaySettingsNormal<IOpenVpnConstraints, IWireguardConstraints> {
  if (relaySettings) {
    const openvpnPort = wrapConstraint(relaySettings.openvpn.port);
    const openvpnProtocol = wrapConstraint(relaySettings.openvpn.protocol);
    const wgPort = wrapConstraint(relaySettings.wireguard.port);
    const wgIpVersion = wrapConstraint(relaySettings.wireguard.ipVersion);
    const wgEntryLocation = wrapConstraint(relaySettings.wireguard.entryLocation);
    const location = wrapConstraint(relaySettings.location);
    const tunnelProtocol = relaySettings.tunnelProtocol;

    return {
      providers: [...relaySettings.providers],
      ownership: relaySettings.ownership,
      tunnelProtocol,
      openvpnConstraints: {
        port: openvpnPort,
        protocol: openvpnProtocol,
      },
      wireguardConstraints: {
        port: wgPort,
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

type RelaySettingsUpdateFunction = (
  settings: IRelaySettingsNormal<IOpenVpnConstraints, IWireguardConstraints>,
) => IRelaySettingsNormal<IOpenVpnConstraints, IWireguardConstraints>;

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

export function wrapBridgeSettingsOrDefault(bridgeSettings?: BridgeSettingsRedux): BridgeSettings {
  if (bridgeSettings) {
    return {
      type: bridgeSettings.type,
      normal: wrapNormalBridgeSettingsOrDefault(bridgeSettings.normal),
      custom: bridgeSettings.custom,
    };
  }

  return {
    type: 'normal',
    normal: wrapNormalBridgeSettingsOrDefault(),
  };
}

function wrapNormalBridgeSettingsOrDefault(
  bridgeSettings?: NormalBridgeSettingsRedux,
): IBridgeConstraints {
  if (bridgeSettings) {
    const location = wrapConstraint(bridgeSettings.location);

    return {
      location,
      providers: [...bridgeSettings.providers],
      ownership: bridgeSettings.ownership,
    };
  }

  return {
    location: 'any',
    providers: [],
    ownership: Ownership.any,
  };
}

type BridgeSettingsUpdateFunction = (settings: BridgeSettings) => BridgeSettings;

export function useBridgeSettingsModifier() {
  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);

  return useCallback(
    (fn: BridgeSettingsUpdateFunction) => {
      const settings = wrapBridgeSettingsOrDefault(bridgeSettings);
      return fn(settings);
    },
    [bridgeSettings],
  );
}

export function useBridgeSettingsUpdater() {
  const { updateBridgeSettings } = useAppContext();
  const modifyBridgeSettings = useBridgeSettingsModifier();

  return useCallback(
    async (fn: BridgeSettingsUpdateFunction) => {
      const modifiedSettings = modifyBridgeSettings(fn);
      await updateBridgeSettings(modifiedSettings);
    },
    [updateBridgeSettings, modifyBridgeSettings],
  );
}
