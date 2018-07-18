// @flow

import type { ReduxAction } from '../store';
import type { RelayProtocol, RelayLocation } from '../../lib/daemon-rpc';

export type RelaySettingsRedux =
  | {|
      normal: {
        location: 'any' | RelayLocation,
        port: 'any' | number,
        protocol: 'any' | RelayProtocol,
      },
    |}
  | {|
      custom_tunnel_endpoint: {
        host: string,
        port: number,
        protocol: RelayProtocol,
      },
    |};

export type RelayLocationCityRedux = {
  name: string,
  code: string,
  latitude: number,
  longitude: number,
  hasActiveRelays: boolean,
};

export type RelayLocationRedux = {
  name: string,
  code: string,
  hasActiveRelays: boolean,
  cities: Array<RelayLocationCityRedux>,
};

export type SettingsReduxState = {
  relaySettings: RelaySettingsRedux,
  relayLocations: Array<RelayLocationRedux>,
  autoConnect: boolean,
  allowLan: boolean,
};

const initialState: SettingsReduxState = {
  relaySettings: {
    normal: {
      location: 'any',
      port: 'any',
      protocol: 'any',
    },
  },
  relayLocations: [],
  autoConnect: false,
  allowLan: false,
};

export default function(
  state: SettingsReduxState = initialState,
  action: ReduxAction,
): SettingsReduxState {
  switch (action.type) {
    case 'UPDATE_RELAY':
      return {
        ...state,
        relaySettings: action.relay,
      };

    case 'UPDATE_RELAY_LOCATIONS':
      return {
        ...state,
        relayLocations: action.relayLocations,
      };

    case 'UPDATE_ALLOW_LAN':
      return {
        ...state,
        allowLan: action.allowLan,
      };

    case 'UPDATE_AUTO_CONNECT':
      return {
        ...state,
        autoConnect: action.autoConnect,
      };

    default:
      return state;
  }
}
