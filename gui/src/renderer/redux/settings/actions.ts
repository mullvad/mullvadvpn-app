import { IWindowsApplication } from '../../../shared/application-types';
import {
  BridgeState,
  CustomLists,
  IDnsOptions,
  IWireguardEndpointData,
  ObfuscationSettings,
} from '../../../shared/daemon-rpc-types';
import { IGuiSettingsState } from '../../../shared/gui-settings-state';
import { BridgeSettingsRedux, IRelayLocationCountryRedux, RelaySettingsRedux } from './reducers';

export interface IUpdateGuiSettingsAction {
  type: 'UPDATE_GUI_SETTINGS';
  guiSettings: IGuiSettingsState;
}

export interface IUpdateRelayAction {
  type: 'UPDATE_RELAY';
  relay: RelaySettingsRedux;
}

export interface IUpdateRelayLocationsAction {
  type: 'UPDATE_RELAY_LOCATIONS';
  relayLocations: IRelayLocationCountryRedux[];
}

export interface IUpdateWireguardEndpointData {
  type: 'UPDATE_WIREGUARD_ENDPOINT_DATA';
  wireguardEndpointData: IWireguardEndpointData;
}

export interface IUpdateAllowLanAction {
  type: 'UPDATE_ALLOW_LAN';
  allowLan: boolean;
}

export interface IUpdateEnableIpv6Action {
  type: 'UPDATE_ENABLE_IPV6';
  enableIpv6: boolean;
}

export interface IUpdateBlockWhenDisconnectedAction {
  type: 'UPDATE_BLOCK_WHEN_DISCONNECTED';
  blockWhenDisconnected: boolean;
}

export interface IUpdateShowBetaReleasesAction {
  type: 'UPDATE_SHOW_BETA_NOTIFICATIONS';
  showBetaReleases: boolean;
}

export interface IUpdateBridgeSettingsAction {
  type: 'UPDATE_BRIDGE_SETTINGS';
  bridgeSettings: BridgeSettingsRedux;
}

export interface IUpdateBridgeStateAction {
  type: 'UPDATE_BRIDGE_STATE';
  bridgeState: BridgeState;
}

export interface IUpdateOpenVpnMssfixAction {
  type: 'UPDATE_OPENVPN_MSSFIX';
  mssfix?: number;
}

export interface IUpdateWireguardMtuAction {
  type: 'UPDATE_WIREGUARD_MTU';
  mtu?: number;
}

export interface IUpdateWireguardQuantumResistantAction {
  type: 'UPDATE_WIREGUARD_QUANTUM_RESISTANT';
  quantumResistant?: boolean;
}

export interface IUpdateAutoStartAction {
  type: 'UPDATE_AUTO_START';
  autoStart: boolean;
}

export interface IUpdateDnsOptionsAction {
  type: 'UPDATE_DNS_OPTIONS';
  dns: IDnsOptions;
}

export interface IUpdateSplitTunnelingStateAction {
  type: 'UPDATE_SPLIT_TUNNELING_STATE';
  enabled: boolean;
}

export interface ISetSplitTunnelingApplicationsAction {
  type: 'SET_SPLIT_TUNNELING_APPLICATIONS';
  applications: IWindowsApplication[];
}

export interface ISetObfuscationSettings {
  type: 'SET_OBFUSCATION_SETTINGS';
  obfuscationSettings: ObfuscationSettings;
}

export interface ISetCustomLists {
  type: 'SET_CUSTOM_LISTS';
  customLists: CustomLists;
}

export type SettingsAction =
  | IUpdateGuiSettingsAction
  | IUpdateRelayAction
  | IUpdateRelayLocationsAction
  | IUpdateWireguardEndpointData
  | IUpdateAllowLanAction
  | IUpdateEnableIpv6Action
  | IUpdateBlockWhenDisconnectedAction
  | IUpdateShowBetaReleasesAction
  | IUpdateBridgeSettingsAction
  | IUpdateBridgeStateAction
  | IUpdateOpenVpnMssfixAction
  | IUpdateWireguardMtuAction
  | IUpdateWireguardQuantumResistantAction
  | IUpdateAutoStartAction
  | IUpdateDnsOptionsAction
  | IUpdateSplitTunnelingStateAction
  | ISetSplitTunnelingApplicationsAction
  | ISetObfuscationSettings
  | ISetCustomLists;

function updateGuiSettings(guiSettings: IGuiSettingsState): IUpdateGuiSettingsAction {
  return {
    type: 'UPDATE_GUI_SETTINGS',
    guiSettings,
  };
}

function updateRelay(relay: RelaySettingsRedux): IUpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay,
  };
}

function updateRelayLocations(
  relayLocations: IRelayLocationCountryRedux[],
): IUpdateRelayLocationsAction {
  return {
    type: 'UPDATE_RELAY_LOCATIONS',
    relayLocations,
  };
}

function updateWireguardEndpointData(
  wireguardEndpointData: IWireguardEndpointData,
): IUpdateWireguardEndpointData {
  return {
    type: 'UPDATE_WIREGUARD_ENDPOINT_DATA',
    wireguardEndpointData,
  };
}

function updateAllowLan(allowLan: boolean): IUpdateAllowLanAction {
  return {
    type: 'UPDATE_ALLOW_LAN',
    allowLan,
  };
}

function updateEnableIpv6(enableIpv6: boolean): IUpdateEnableIpv6Action {
  return {
    type: 'UPDATE_ENABLE_IPV6',
    enableIpv6,
  };
}

function updateBlockWhenDisconnected(
  blockWhenDisconnected: boolean,
): IUpdateBlockWhenDisconnectedAction {
  return {
    type: 'UPDATE_BLOCK_WHEN_DISCONNECTED',
    blockWhenDisconnected,
  };
}

function updateShowBetaReleases(showBetaReleases: boolean): IUpdateShowBetaReleasesAction {
  return {
    type: 'UPDATE_SHOW_BETA_NOTIFICATIONS',
    showBetaReleases,
  };
}

function updateBridgeSettings(bridgeSettings: BridgeSettingsRedux): IUpdateBridgeSettingsAction {
  return {
    type: 'UPDATE_BRIDGE_SETTINGS',
    bridgeSettings,
  };
}

function updateBridgeState(bridgeState: BridgeState): IUpdateBridgeStateAction {
  return {
    type: 'UPDATE_BRIDGE_STATE',
    bridgeState,
  };
}

function updateOpenVpnMssfix(mssfix?: number): IUpdateOpenVpnMssfixAction {
  return {
    type: 'UPDATE_OPENVPN_MSSFIX',
    mssfix,
  };
}

function updateWireguardMtu(mtu?: number): IUpdateWireguardMtuAction {
  return {
    type: 'UPDATE_WIREGUARD_MTU',
    mtu,
  };
}

function updateWireguardQuantumResistant(
  quantumResistant?: boolean,
): IUpdateWireguardQuantumResistantAction {
  return {
    type: 'UPDATE_WIREGUARD_QUANTUM_RESISTANT',
    quantumResistant,
  };
}

function updateAutoStart(autoStart: boolean): IUpdateAutoStartAction {
  return {
    type: 'UPDATE_AUTO_START',
    autoStart,
  };
}

function updateDnsOptions(dns: IDnsOptions): IUpdateDnsOptionsAction {
  return {
    type: 'UPDATE_DNS_OPTIONS',
    dns,
  };
}

function updateSplitTunnelingState(enabled: boolean): IUpdateSplitTunnelingStateAction {
  return {
    type: 'UPDATE_SPLIT_TUNNELING_STATE',
    enabled,
  };
}

function setSplitTunnelingApplications(
  applications: IWindowsApplication[],
): ISetSplitTunnelingApplicationsAction {
  return {
    type: 'SET_SPLIT_TUNNELING_APPLICATIONS',
    applications,
  };
}

function updateObfuscationSettings(
  obfuscationSettings: ObfuscationSettings,
): ISetObfuscationSettings {
  return {
    type: 'SET_OBFUSCATION_SETTINGS',
    obfuscationSettings,
  };
}

function updateCustomLists(customLists: CustomLists): ISetCustomLists {
  return {
    type: 'SET_CUSTOM_LISTS',
    customLists,
  };
}

export default {
  updateGuiSettings,
  updateRelay,
  updateRelayLocations,
  updateWireguardEndpointData,
  updateAllowLan,
  updateEnableIpv6,
  updateBlockWhenDisconnected,
  updateShowBetaReleases,
  updateBridgeSettings,
  updateBridgeState,
  updateOpenVpnMssfix,
  updateWireguardMtu,
  updateWireguardQuantumResistant,
  updateAutoStart,
  updateDnsOptions,
  updateSplitTunnelingState,
  setSplitTunnelingApplications,
  updateObfuscationSettings,
  updateCustomLists,
};
