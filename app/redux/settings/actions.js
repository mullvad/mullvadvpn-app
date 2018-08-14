// @flow

import type { RelaySettingsRedux, RelayLocationRedux } from './reducers';

export type UpdateRelayAction = {
  type: 'UPDATE_RELAY',
  relay: RelaySettingsRedux,
};

export type UpdateRelayLocationsAction = {
  type: 'UPDATE_RELAY_LOCATIONS',
  relayLocations: Array<RelayLocationRedux>,
};

export type UpdateAutoConnectAction = {
  type: 'UPDATE_AUTO_CONNECT',
  autoConnect: boolean,
};

export type UpdateAllowLanAction = {
  type: 'UPDATE_ALLOW_LAN',
  allowLan: boolean,
};

export type UpdateEnableIpv6Action = {
  type: 'UPDATE_ENABLE_IPV6',
  enableIpv6: boolean,
};

export type SettingsAction =
  | UpdateRelayAction
  | UpdateRelayLocationsAction
  | UpdateAutoConnectAction
  | UpdateAllowLanAction
  | UpdateEnableIpv6Action;

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

function updateAutoConnect(autoConnect: boolean): UpdateAutoConnectAction {
  return {
    type: 'UPDATE_AUTO_CONNECT',
    autoConnect,
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

export default {
  updateRelay,
  updateRelayLocations,
  updateAutoConnect,
  updateAllowLan,
  updateEnableIpv6,
};
