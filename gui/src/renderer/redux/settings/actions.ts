import {
  BridgeState,
  IDnsOptions,
  IWireguardPublicKey,
  KeygenEvent,
} from '../../../shared/daemon-rpc-types';
import { IGuiSettingsState } from '../../../shared/gui-settings-state';
import { IApplication } from '../../../shared/application-types';
import { BridgeSettingsRedux, IRelayLocationRedux, IWgKey, RelaySettingsRedux } from './reducers';

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
  relayLocations: IRelayLocationRedux[];
}

export interface IUpdateBridgeLocationsAction {
  type: 'UPDATE_BRIDGE_LOCATIONS';
  bridgeLocations: IRelayLocationRedux[];
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

export interface IUpdateAutoStartAction {
  type: 'UPDATE_AUTO_START';
  autoStart: boolean;
}

// Used to set wireguard key when accounts are changed.
export interface IWireguardSetKey {
  type: 'SET_WIREGUARD_KEY';
  key?: IWgKey;
}

export interface IWireguardGenerateKey {
  type: 'GENERATE_WIREGUARD_KEY';
}

export interface IWireguardReplaceKey {
  type: 'REPLACE_WIREGUARD_KEY';
  oldKey: IWgKey;
}

export interface IWireguardVerifyKey {
  type: 'VERIFY_WIREGUARD_KEY';
  key: IWgKey;
}

export interface IWireguardKeygenEvent {
  type: 'WIREGUARD_KEYGEN_EVENT';
  event: KeygenEvent;
}

export interface IWireguardKeyVerifiedAction {
  type: 'WIREGUARD_KEY_VERIFICATION_COMPLETE';
  verified?: boolean;
}

export interface IUpdateDnsOptionsAction {
  type: 'UPDATE_DNS_OPTIONS';
  dns: IDnsOptions;
}

export interface ISplitTunnelingEnableExclusions {
  type: 'SPLIT_TUNNELING_ENABLE_EXCLUSIONS';
  enabled: boolean;
}

export interface ISplitTunnelingApplications {
  type: 'SPLIT_TUNNELING_APPLICATIONS';
  applications: IApplication[];
}

export type SettingsAction =
  | IUpdateGuiSettingsAction
  | IUpdateRelayAction
  | IUpdateRelayLocationsAction
  | IUpdateBridgeLocationsAction
  | IUpdateAllowLanAction
  | IUpdateEnableIpv6Action
  | IUpdateBlockWhenDisconnectedAction
  | IUpdateShowBetaReleasesAction
  | IUpdateBridgeSettingsAction
  | IUpdateBridgeStateAction
  | IUpdateOpenVpnMssfixAction
  | IUpdateWireguardMtuAction
  | IUpdateAutoStartAction
  | IWireguardSetKey
  | IWireguardVerifyKey
  | IWireguardGenerateKey
  | IWireguardReplaceKey
  | IWireguardKeygenEvent
  | IWireguardKeyVerifiedAction
  | IUpdateDnsOptionsAction
  | ISplitTunnelingEnableExclusions
  | ISplitTunnelingApplications;

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

function updateRelayLocations(relayLocations: IRelayLocationRedux[]): IUpdateRelayLocationsAction {
  return {
    type: 'UPDATE_RELAY_LOCATIONS',
    relayLocations,
  };
}

function updateBridgeLocations(
  bridgeLocations: IRelayLocationRedux[],
): IUpdateBridgeLocationsAction {
  return {
    type: 'UPDATE_BRIDGE_LOCATIONS',
    bridgeLocations,
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

function updateAutoStart(autoStart: boolean): IUpdateAutoStartAction {
  return {
    type: 'UPDATE_AUTO_START',
    autoStart,
  };
}

function setWireguardKey(publicKey?: IWireguardPublicKey): IWireguardSetKey {
  const key = publicKey
    ? {
        publicKey: publicKey.key,
        created: publicKey.created,
        valid: undefined,
      }
    : undefined;
  return {
    type: 'SET_WIREGUARD_KEY',
    key,
  };
}

function setWireguardKeygenEvent(event: KeygenEvent): IWireguardKeygenEvent {
  return {
    type: 'WIREGUARD_KEYGEN_EVENT',
    event,
  };
}

function generateWireguardKey(): IWireguardGenerateKey {
  return {
    type: 'GENERATE_WIREGUARD_KEY',
  };
}

function replaceWireguardKey(oldKey: IWgKey): IWireguardReplaceKey {
  return {
    type: 'REPLACE_WIREGUARD_KEY',
    oldKey,
  };
}

function verifyWireguardKey(key: IWgKey): IWireguardVerifyKey {
  return {
    type: 'VERIFY_WIREGUARD_KEY',
    key,
  };
}

function completeWireguardKeyVerification(verified?: boolean): IWireguardKeyVerifiedAction {
  return {
    type: 'WIREGUARD_KEY_VERIFICATION_COMPLETE',
    verified,
  };
}

function updateDnsOptions(dns: IDnsOptions): IUpdateDnsOptionsAction {
  return {
    type: 'UPDATE_DNS_OPTIONS',
    dns,
  };
}

function updateSplitTunneling(enabled: boolean): ISplitTunnelingEnableExclusions {
  return {
    type: 'SPLIT_TUNNELING_ENABLE_EXCLUSIONS',
    enabled,
  };
}

function setSplitTunnelingApplications(applications: IApplication[]): ISplitTunnelingApplications {
  return {
    type: 'SPLIT_TUNNELING_APPLICATIONS',
    applications,
  };
}

export default {
  updateGuiSettings,
  updateRelay,
  updateRelayLocations,
  updateBridgeLocations,
  updateAllowLan,
  updateEnableIpv6,
  updateBlockWhenDisconnected,
  updateShowBetaReleases,
  updateBridgeSettings,
  updateBridgeState,
  updateOpenVpnMssfix,
  updateWireguardMtu,
  updateAutoStart,
  setWireguardKey,
  setWireguardKeygenEvent,
  generateWireguardKey,
  replaceWireguardKey,
  verifyWireguardKey,
  completeWireguardKeyVerification,
  updateDnsOptions,
  updateSplitTunneling,
  setSplitTunnelingApplications,
};
