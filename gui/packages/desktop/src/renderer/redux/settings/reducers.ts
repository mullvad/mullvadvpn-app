import { ReduxAction } from '../store';
import { RelayProtocol, RelayLocation } from '../../../shared/daemon-rpc-types';
import { GuiSettingsState } from '../../../shared/gui-settings-state';

export type RelaySettingsRedux =
  | {
      normal: {
        location: 'any' | RelayLocation;
        port: 'any' | number;
        protocol: 'any' | RelayProtocol;
      };
    }
  | {
      customTunnelEndpoint: {
        host: string;
        port: number;
        protocol: RelayProtocol;
      };
    };

export type RelayLocationRelayRedux = {
  hostname: string;
  ipv4AddrIn: string;
  ipv4AddrExit: string;
  includeInCountry: boolean;
  weight: number;
};

export type RelayLocationCityRedux = {
  name: string;
  code: string;
  latitude: number;
  longitude: number;
  hasActiveRelays: boolean;
  relays: RelayLocationRelayRedux[];
};

export type RelayLocationRedux = {
  name: string;
  code: string;
  hasActiveRelays: boolean;
  cities: RelayLocationCityRedux[];
};

export type SettingsReduxState = {
  autoStart: boolean;
  guiSettings: GuiSettingsState;
  relaySettings: RelaySettingsRedux;
  relayLocations: RelayLocationRedux[];
  allowLan: boolean;
  enableIpv6: boolean;
  blockWhenDisconnected: boolean;
  openVpn: {
    mssfix?: number;
  };
};

const initialState: SettingsReduxState = {
  autoStart: false,
  guiSettings: {
    autoConnect: true,
    monochromaticIcon: false,
    startMinimized: false,
  },
  relaySettings: {
    normal: {
      location: 'any',
      port: 'any',
      protocol: 'any',
    },
  },
  relayLocations: [],
  allowLan: false,
  enableIpv6: true,
  blockWhenDisconnected: false,
  openVpn: {},
};

export default function(
  state: SettingsReduxState = initialState,
  action: ReduxAction,
): SettingsReduxState {
  switch (action.type) {
    case 'UPDATE_GUI_SETTINGS':
      return {
        ...state,
        guiSettings: action.guiSettings,
      };

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

    case 'UPDATE_ENABLE_IPV6':
      return {
        ...state,
        enableIpv6: action.enableIpv6,
      };

    case 'UPDATE_BLOCK_WHEN_DISCONNECTED':
      return {
        ...state,
        blockWhenDisconnected: action.blockWhenDisconnected,
      };

    case 'UPDATE_OPENVPN_MSSFIX':
      return {
        ...state,
        openVpn: {
          ...state.openVpn,
          mssfix: action.mssfix,
        },
      };

    case 'UPDATE_AUTO_START':
      return {
        ...state,
        autoStart: action.autoStart,
      };

    default:
      return state;
  }
}
