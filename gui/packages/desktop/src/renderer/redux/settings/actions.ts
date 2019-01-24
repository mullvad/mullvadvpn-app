import { RelaySettingsRedux, RelayLocationRedux } from './reducers';
import { GuiSettingsState } from '../../../shared/gui-settings-state';

export type UpdateGuiSettingsAction = {
  type: 'UPDATE_GUI_SETTINGS';
  guiSettings: GuiSettingsState;
};

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY';
  relay: RelaySettingsRedux;
};

export type UpdateRelayLocationsAction = {
  type: 'UPDATE_RELAY_LOCATIONS';
  relayLocations: Array<RelayLocationRedux>;
};

export type UpdateAllowLanAction = {
  type: 'UPDATE_ALLOW_LAN';
  allowLan: boolean;
};

export type UpdateEnableIpv6Action = {
  type: 'UPDATE_ENABLE_IPV6';
  enableIpv6: boolean;
};

export type UpdateBlockWhenDisconnectedAction = {
  type: 'UPDATE_BLOCK_WHEN_DISCONNECTED';
  blockWhenDisconnected: boolean;
};

export type UpdateOpenVpnMssfixAction = {
  type: 'UPDATE_OPENVPN_MSSFIX';
  mssfix?: number;
};

export type UpdateAutoStartAction = {
  type: 'UPDATE_AUTO_START';
  autoStart: boolean;
};

export type SettingsAction =
  | UpdateGuiSettingsAction
  | UpdateRelayAction
  | UpdateRelayLocationsAction
  | UpdateAllowLanAction
  | UpdateEnableIpv6Action
  | UpdateBlockWhenDisconnectedAction
  | UpdateOpenVpnMssfixAction
  | UpdateAutoStartAction;

function updateGuiSettings(guiSettings: GuiSettingsState): UpdateGuiSettingsAction {
  return {
    type: 'UPDATE_GUI_SETTINGS',
    guiSettings,
  };
}

function updateRelay(relay: RelaySettingsRedux): UpdateRelayAction {
  return {
    type: 'UPDATE_RELAY',
    relay: relay,
  };
}

function updateRelayLocations(
  relayLocations: Array<RelayLocationRedux>,
): UpdateRelayLocationsAction {
  return {
    type: 'UPDATE_RELAY_LOCATIONS',
    relayLocations: relayLocations,
  };
}

function updateAllowLan(allowLan: boolean): UpdateAllowLanAction {
  return {
    type: 'UPDATE_ALLOW_LAN',
    allowLan,
  };
}

function updateEnableIpv6(enableIpv6: boolean): UpdateEnableIpv6Action {
  return {
    type: 'UPDATE_ENABLE_IPV6',
    enableIpv6,
  };
}

function updateBlockWhenDisconnected(
  blockWhenDisconnected: boolean,
): UpdateBlockWhenDisconnectedAction {
  return {
    type: 'UPDATE_BLOCK_WHEN_DISCONNECTED',
    blockWhenDisconnected,
  };
}

function updateOpenVpnMssfix(mssfix?: number): UpdateOpenVpnMssfixAction {
  return {
    type: 'UPDATE_OPENVPN_MSSFIX',
    mssfix,
  };
}

function updateAutoStart(autoStart: boolean): UpdateAutoStartAction {
  return {
    type: 'UPDATE_AUTO_START',
    autoStart,
  };
}

export default {
  updateGuiSettings,
  updateRelay,
  updateRelayLocations,
  updateAllowLan,
  updateEnableIpv6,
  updateBlockWhenDisconnected,
  updateOpenVpnMssfix,
  updateAutoStart,
};
