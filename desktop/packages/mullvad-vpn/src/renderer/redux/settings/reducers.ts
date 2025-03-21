import { getDefaultApiAccessMethods } from '../../../main/default-settings';
import { ISplitTunnelingApplication } from '../../../shared/application-types';
import {
  AccessMethodSetting,
  ApiAccessMethodSettings,
  BridgeState,
  BridgeType,
  CustomLists,
  CustomProxy,
  IDaitaSettings,
  IDnsOptions,
  IpVersion,
  IWireguardEndpointData,
  LiftedConstraint,
  ObfuscationSettings,
  ObfuscationType,
  Ownership,
  RelayEndpointType,
  RelayLocation,
  RelayOverride,
  RelayProtocol,
  TunnelProtocol,
} from '../../../shared/daemon-rpc-types';
import { IGuiSettingsState } from '../../../shared/gui-settings-state';
import { ReduxAction } from '../store';

export type NormalRelaySettingsRedux = {
  tunnelProtocol: TunnelProtocol;
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

export type NormalBridgeSettingsRedux = {
  location: LiftedConstraint<RelayLocation>;
  /** Providers are used to filter bridges and as bridge constraints for the daemon. */
  providers: string[];
  /** Ownership is used to filter bridges and as bridge constraints for the daemon. */
  ownership: Ownership;
};

export type RelaySettingsRedux =
  | {
      normal: NormalRelaySettingsRedux;
    }
  | {
      customTunnelEndpoint: {
        host: string;
        port: number;
        protocol: RelayProtocol;
      };
    };

export type BridgeSettingsRedux = {
  type: BridgeType;
  normal: NormalBridgeSettingsRedux;
  custom?: CustomProxy;
};

export interface IRelayLocationRelayRedux {
  hostname: string;
  provider: string;
  ipv4AddrIn: string;
  includeInCountry: boolean;
  active: boolean;
  owned: boolean;
  weight: number;
  endpointType: RelayEndpointType;
  daita: boolean;
}

export interface IRelayLocationCityRedux {
  name: string;
  code: string;
  latitude: number;
  longitude: number;
  relays: IRelayLocationRelayRedux[];
}

export interface IRelayLocationCountryRedux {
  name: string;
  code: string;
  cities: IRelayLocationCityRedux[];
}

export interface ISettingsReduxState {
  autoStart: boolean;
  guiSettings: IGuiSettingsState;
  relaySettings: RelaySettingsRedux;
  relayLocations: IRelayLocationCountryRedux[];
  wireguardEndpointData: IWireguardEndpointData;
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
    quantumResistant?: boolean;
    daita?: IDaitaSettings;
  };
  dns: IDnsOptions;
  splitTunneling: boolean;
  splitTunnelingApplications: ISplitTunnelingApplication[];
  obfuscationSettings: ObfuscationSettings;
  customLists: CustomLists;
  apiAccessMethods: ApiAccessMethodSettings;
  currentApiAccessMethod?: AccessMethodSetting;
  relayOverrides: Array<RelayOverride>;
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
    updateDismissedForVersion: '',
    animateMap: true,
  },
  relaySettings: {
    normal: {
      location: 'any',
      tunnelProtocol: 'wireguard',
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
  wireguardEndpointData: { portRanges: [], udp2tcpPorts: [] },
  allowLan: false,
  enableIpv6: true,
  bridgeSettings: {
    type: 'normal',
    normal: {
      location: 'any',
      providers: [],
      ownership: Ownership.any,
    },
    custom: undefined,
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
      blockSocialMedia: false,
    },
    customOptions: {
      addresses: [],
    },
  },
  splitTunneling: false,
  splitTunnelingApplications: [],
  obfuscationSettings: {
    selectedObfuscation: ObfuscationType.auto,
    udp2tcpSettings: {
      port: 'any',
    },
    shadowsocksSettings: {
      port: 'any',
    },
  },
  customLists: [],
  apiAccessMethods: getDefaultApiAccessMethods(),
  currentApiAccessMethod: undefined,
  relayOverrides: [],
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

    case 'UPDATE_WIREGUARD_ENDPOINT_DATA':
      return {
        ...state,
        wireguardEndpointData: action.wireguardEndpointData,
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

    case 'UPDATE_WIREGUARD_QUANTUM_RESISTANT':
      return {
        ...state,
        wireguard: {
          ...state.wireguard,
          quantumResistant: action.quantumResistant,
        },
      };
    case 'UPDATE_WIREGUARD_DAITA':
      return {
        ...state,
        wireguard: {
          ...state.wireguard,
          daita: action.daita,
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

    case 'SET_OBFUSCATION_SETTINGS':
      return {
        ...state,
        obfuscationSettings: action.obfuscationSettings,
      };

    case 'SET_CUSTOM_LISTS':
      return {
        ...state,
        customLists: action.customLists,
      };

    case 'SET_API_ACCESS_METHODS':
      return {
        ...state,
        apiAccessMethods: action.accessMethods,
      };

    case 'SET_CURRENT_API_ACCESS_METHOD':
      return {
        ...state,
        currentApiAccessMethod: action.accessMethod,
      };

    case 'SET_RELAY_OVERRIDES':
      return {
        ...state,
        relayOverrides: action.relayOverrides,
      };

    default:
      return state;
  }
}
