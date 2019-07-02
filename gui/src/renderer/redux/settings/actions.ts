import { BridgeState, KeygenEvent } from '../../../shared/daemon-rpc-types';
import { IGuiSettingsState } from '../../../shared/gui-settings-state';
import { IRelayLocationRedux, RelaySettingsRedux } from './reducers';

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

export interface IUpdateBridgeStateAction {
  type: 'UPDATE_BRIDGE_STATE';
  bridgeState: BridgeState;
}

export interface IUpdateOpenVpnMssfixAction {
  type: 'UPDATE_OPENVPN_MSSFIX';
  mssfix?: number;
}

export interface IUpdateAutoStartAction {
  type: 'UPDATE_AUTO_START';
  autoStart: boolean;
}

// Used to set wireguard key when accounts are changed.
export interface IWireguardSetKey {
  type: 'SET_WIREGUARD_KEY';
  publicKey?: string;
}

export interface IWireguardGenerateKey {
  type: 'GENERATE_WIREGUARD_KEY';
}

export interface IWireguardVerifyKey {
  type: 'VERIFY_WIREGUARD_KEY';
  publicKey: string;
}

export interface IWireguardKeygenEvent {
  type: 'WIREGUARD_KEYGEN_EVENT';
  event: KeygenEvent;
}

export interface IWireguardKeyVerifiedAction {
  type: 'WIREGUARD_KEY_VERIFICATION_COMPLETE';
  verified: boolean;
}

export type SettingsAction =
  | IUpdateGuiSettingsAction
  | IUpdateRelayAction
  | IUpdateRelayLocationsAction
  | IUpdateAllowLanAction
  | IUpdateEnableIpv6Action
  | IUpdateBlockWhenDisconnectedAction
  | IUpdateBridgeStateAction
  | IUpdateOpenVpnMssfixAction
  | IUpdateAutoStartAction
  | IWireguardSetKey
  | IWireguardVerifyKey
  | IWireguardGenerateKey
  | IWireguardKeygenEvent
  | IWireguardKeyVerifiedAction;

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

function updateAutoStart(autoStart: boolean): IUpdateAutoStartAction {
  return {
    type: 'UPDATE_AUTO_START',
    autoStart,
  };
}

function setWireguardKey(publicKey?: string): IWireguardSetKey {
  return {
    type: 'SET_WIREGUARD_KEY',
    publicKey,
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

function verifyWireguardKey(publicKey: string): IWireguardVerifyKey {
  return {
    type: 'VERIFY_WIREGUARD_KEY',
    publicKey,
  };
}

function completeWireguardKeyVerification(verified: boolean): IWireguardKeyVerifiedAction {
  return {
    type: 'WIREGUARD_KEY_VERIFICATION_COMPLETE',
    verified,
  };
}

export default {
  updateGuiSettings,
  updateRelay,
  updateRelayLocations,
  updateAllowLan,
  updateEnableIpv6,
  updateBlockWhenDisconnected,
  updateBridgeState,
  updateOpenVpnMssfix,
  updateAutoStart,
  setWireguardKey,
  setWireguardKeygenEvent,
  generateWireguardKey,
  verifyWireguardKey,
  completeWireguardKeyVerification,
};
