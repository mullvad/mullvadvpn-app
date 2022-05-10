import { IWindowsApplication } from '../../../shared/application-types';
import {
  BridgeState,
  IDnsOptions,
  IpVersion,
  LiftedConstraint,
  Ownership,
  ProxySettings,
  RelayLocation,
  RelayProtocol,
  TunnelProtocol,
} from '../../../shared/daemon-rpc-types';
import { IGuiSettingsState } from '../../../shared/gui-settings-state';
import { ReduxAction } from '../store';

export type RelaySettingsRedux =
  | {
      normal: {
        tunnelProtocol: LiftedConstraint<TunnelProtocol>;
        location: LiftedConstraint<RelayLocation>;
        providers: string[];
        ownership: Ownership;
        openvpn: {
          port: LiftedConstraint<number>;
          protocol: LiftedConstraint<RelayProtocol>;
        };
        wireguard: {
          port: LiftedConstraint<number>;
          ipVersion: LiftedConstraint<IpVersion>;
          useMultihop: boolean;
          entryLocation: LiftedConstraint<RelayLocation>;
        };
      };
    }
  | {
      customTunnelEndpoint: {
        host: string;
        port: number;
        protocol: RelayProtocol;
      };
    };

export type BridgeSettingsRedux =
  | {
      normal: {
        location: LiftedConstraint<RelayLocation>;
      };
    }
  | {
      custom: ProxySettings;
    };

export interface IRelayLocationRelayRedux {
  hostname: string;
  provider: string;
  ipv4AddrIn: string;
  includeInCountry: boolean;
  active: boolean;
  weight: number;
}

export interface IRelayLocationCityRedux {
  name: string;
  code: string;
  latitude: number;
  longitude: number;
  relays: IRelayLocationRelayRedux[];
}

export interface IRelayLocationRedux {
  name: string;
  code: string;
  cities: IRelayLocationCityRedux[];
}

export interface ISettingsReduxState {
  autoStart: boolean;
  guiSettings: IGuiSettingsState;
  relaySettings: RelaySettingsRedux;
  relayLocations: IRelayLocationRedux[];
  bridgeLocations: IRelayLocationRedux[];
  allowLan: boolean;
  enableIpv6: boolean;
  bridgeSettings: BridgeSettingsRedux;
  bridgeState: BridgeState;
  blockWhenDisconnected: boolean;
  showBetaReleases: boolean;
  openVpn: {
    mssfix?: number;
  };
  wireguard: {
    mtu?: number;
  };
  dns: IDnsOptions;
  splitTunneling: boolean;
  splitTunnelingApplications: IWindowsApplication[];
}

const initialState: ISettingsReduxState = {
  autoStart: false,
  guiSettings: {
    preferredLocale: 'system',
    enableSystemNotifications: true,
    autoConnect: true,
    monochromaticIcon: false,
    startMinimized: false,
    unpinnedWindow: window.env.platform !== 'win32' && window.env.platform !== 'darwin',
    browsedForSplitTunnelingApplications: [],
    changelogDisplayedForVersion: '',
  },
  relaySettings: {
    normal: {
      location: 'any',
      tunnelProtocol: 'any',
      providers: [],
      ownership: Ownership.any,
      wireguard: { port: 'any', ipVersion: 'any', useMultihop: false, entryLocation: 'any' },
      openvpn: {
        port: 'any',
        protocol: 'any',
      },
    },
  },
  relayLocations: [],
  bridgeLocations: [],
  allowLan: false,
  enableIpv6: true,
  bridgeSettings: {
    normal: {
      location: 'any',
    },
  },
  bridgeState: 'auto',
  blockWhenDisconnected: false,
  showBetaReleases: false,
  openVpn: {},
  wireguard: {},
  dns: {
    state: 'default',
    defaultOptions: {
      blockAds: false,
      blockTrackers: false,
      blockMalware: false,
      blockAdultContent: false,
      blockGambling: false,
    },
    customOptions: {
      addresses: [],
    },
  },
  splitTunneling: false,
  splitTunnelingApplications: [],
};

export default function (
  state: ISettingsReduxState = initialState,
  action: ReduxAction,
): ISettingsReduxState {
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

    case 'UPDATE_BRIDGE_LOCATIONS':
      return {
        ...state,
        bridgeLocations: action.bridgeLocations,
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

    case 'UPDATE_SHOW_BETA_NOTIFICATIONS':
      return {
        ...state,
        showBetaReleases: action.showBetaReleases,
      };

    case 'UPDATE_OPENVPN_MSSFIX':
      return {
        ...state,
        openVpn: {
          ...state.openVpn,
          mssfix: action.mssfix,
        },
      };

    case 'UPDATE_WIREGUARD_MTU':
      return {
        ...state,
        wireguard: {
          ...state.wireguard,
          mtu: action.mtu,
        },
      };

    case 'UPDATE_AUTO_START':
      return {
        ...state,
        autoStart: action.autoStart,
      };

    case 'UPDATE_BRIDGE_SETTINGS':
      return {
        ...state,
        bridgeSettings: action.bridgeSettings,
      };

    case 'UPDATE_BRIDGE_STATE':
      return {
        ...state,
        bridgeState: action.bridgeState,
      };

    case 'UPDATE_DNS_OPTIONS':
      return {
        ...state,
        dns: action.dns,
      };

    case 'UPDATE_SPLIT_TUNNELING_STATE':
      return {
        ...state,
        splitTunneling: action.enabled,
      };

    case 'SET_SPLIT_TUNNELING_APPLICATIONS':
      return {
        ...state,
        splitTunnelingApplications: action.applications,
      };

    default:
      return state;
  }
}
