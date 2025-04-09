import { types as grpcTypes } from 'management-interface';

import {
  AccessMethod,
  AccessMethodSetting,
  AfterDisconnect,
  ApiAccessMethodSettings,
  AuthFailedError,
  BridgeSettings,
  BridgesMethod,
  BridgeState,
  BridgeType,
  ConnectionConfig,
  Constraint,
  CustomLists,
  CustomProxy,
  DaemonEvent,
  DeviceEvent,
  DeviceState,
  DirectMethod,
  EncryptedDnsProxy,
  EndpointObfuscationType,
  ErrorStateCause,
  ErrorStateDetails,
  FeatureIndicator,
  FirewallPolicyError,
  FirewallPolicyErrorType,
  IBridgeConstraints,
  ICustomList,
  IDevice,
  IObfuscationEndpoint,
  IOpenVpnConstraints,
  IProxyEndpoint,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  IRelayListWithEndpointData,
  IRelaySettingsNormal,
  ISettings,
  ITunnelOptions,
  ITunnelStateRelayInfo,
  IWireguardConstraints,
  IWireguardEndpointData,
  LoggedInDeviceState,
  LoggedOutDeviceState,
  NewAccessMethodSetting,
  ObfuscationSettings,
  ObfuscationType,
  Ownership,
  ProxyType,
  RelayEndpointType,
  RelayLocation,
  RelayLocationGeographical,
  RelayProtocol,
  RelaySettings,
  SocksAuth,
  TunnelParameterError,
  TunnelProtocol,
  TunnelState,
  TunnelType,
  wrapConstraint,
} from '../shared/daemon-rpc-types';

export class ResponseParseError extends Error {
  constructor(message: string) {
    super(message);
  }
}

function unwrapConstraint<T>(constraint: Constraint<T> | undefined): T | undefined {
  if (constraint !== undefined && constraint !== 'any') {
    return constraint.only;
  }
  return undefined;
}

export function convertFromRelayList(relayList: grpcTypes.RelayList): IRelayListWithEndpointData {
  return {
    relayList: {
      countries: relayList
        .getCountriesList()
        .map((country: grpcTypes.RelayListCountry) => convertFromRelayListCountry(country)),
    },
    wireguardEndpointData: convertWireguardEndpointData(relayList.getWireguard()!),
  };
}

function convertWireguardEndpointData(
  data: grpcTypes.WireguardEndpointData,
): IWireguardEndpointData {
  return {
    portRanges: data.getPortRangesList().map((range) => [range.getFirst(), range.getLast()]),
    udp2tcpPorts: data.getUdp2tcpPortsList(),
  };
}

function convertFromRelayListCountry(country: grpcTypes.RelayListCountry): IRelayListCountry {
  const countryObject = country.toObject();
  return {
    ...countryObject,
    cities: country.getCitiesList().map(convertFromRelayListCity),
  };
}

function convertFromRelayListCity(city: grpcTypes.RelayListCity): IRelayListCity {
  const cityObject = city.toObject();
  return {
    ...cityObject,
    relays: city.getRelaysList().map(convertFromRelayListRelay),
  };
}

function convertFromRelayListRelay(relay: grpcTypes.Relay): IRelayListHostname {
  const relayObject = relay.toObject();

  let daita = false;
  if (relayObject.endpointType === grpcTypes.Relay.RelayType.WIREGUARD) {
    const endpointDataU8 = relay.getEndpointData()?.getValue_asU8();
    if (endpointDataU8) {
      daita = grpcTypes.WireguardRelayEndpointData.deserializeBinary(endpointDataU8).getDaita();
    }
  }

  return {
    ...relayObject,
    endpointType: convertFromRelayType(relayObject.endpointType),
    daita,
  };
}

function convertFromRelayType(relayType: grpcTypes.Relay.RelayType): RelayEndpointType {
  const protocolMap: Record<grpcTypes.Relay.RelayType, RelayEndpointType> = {
    [grpcTypes.Relay.RelayType.OPENVPN]: 'openvpn',
    [grpcTypes.Relay.RelayType.BRIDGE]: 'bridge',
    [grpcTypes.Relay.RelayType.WIREGUARD]: 'wireguard',
  };
  return protocolMap[relayType];
}

function convertFromWireguardKey(publicKey: Uint8Array | string): string {
  if (typeof publicKey === 'string') {
    return publicKey;
  }
  return Buffer.from(publicKey).toString('base64');
}

function convertFromTransportProtocol(protocol: grpcTypes.TransportProtocol): RelayProtocol {
  const protocolMap: Record<grpcTypes.TransportProtocol, RelayProtocol> = {
    [grpcTypes.TransportProtocol.TCP]: 'tcp',
    [grpcTypes.TransportProtocol.UDP]: 'udp',
  };
  return protocolMap[protocol];
}

export function convertFromTunnelState(
  tunnelState: grpcTypes.TunnelState,
): TunnelState | undefined {
  const tunnelStateObject = tunnelState.toObject();
  switch (tunnelState.getStateCase()) {
    case grpcTypes.TunnelState.StateCase.STATE_NOT_SET:
      return undefined;
    case grpcTypes.TunnelState.StateCase.DISCONNECTED:
      return {
        state: 'disconnected',
        location: tunnelStateObject.disconnected!.disconnectedLocation,
      };
    case grpcTypes.TunnelState.StateCase.DISCONNECTING: {
      const detailsMap: Record<grpcTypes.AfterDisconnect, AfterDisconnect> = {
        [grpcTypes.AfterDisconnect.NOTHING]: 'nothing',
        [grpcTypes.AfterDisconnect.BLOCK]: 'block',
        [grpcTypes.AfterDisconnect.RECONNECT]: 'reconnect',
      };
      return (
        tunnelStateObject.disconnecting && {
          state: 'disconnecting',
          details: detailsMap[tunnelStateObject.disconnecting.afterDisconnect],
        }
      );
    }
    case grpcTypes.TunnelState.StateCase.ERROR:
      return (
        tunnelStateObject.error?.errorState && {
          state: 'error',
          details: convertFromTunnelStateError(tunnelStateObject.error.errorState),
        }
      );
    case grpcTypes.TunnelState.StateCase.CONNECTING:
      return {
        state: 'connecting',
        details:
          tunnelStateObject.connecting?.relayInfo &&
          convertFromTunnelStateRelayInfo(tunnelStateObject.connecting.relayInfo),
        featureIndicators: convertFromFeatureIndicators(
          tunnelStateObject.connecting?.featureIndicators?.activeFeaturesList,
        ),
      };
    case grpcTypes.TunnelState.StateCase.CONNECTED: {
      const relayInfo =
        tunnelStateObject.connected?.relayInfo &&
        convertFromTunnelStateRelayInfo(tunnelStateObject.connected.relayInfo);
      return (
        relayInfo && {
          state: 'connected',
          details: relayInfo,
          featureIndicators: convertFromFeatureIndicators(
            tunnelStateObject.connected?.featureIndicators?.activeFeaturesList,
          ),
        }
      );
    }
  }
}

function convertFromTunnelStateError(state: grpcTypes.ErrorState.AsObject): ErrorStateDetails {
  const baseError = {
    blockingError: state.blockingError && convertFromBlockingError(state.blockingError),
  };

  switch (state.cause) {
    case grpcTypes.ErrorState.Cause.AUTH_FAILED:
      return {
        ...baseError,
        cause: ErrorStateCause.authFailed,
        authFailedError: convertFromAuthFailedError(state.authFailedError),
      };
    case grpcTypes.ErrorState.Cause.TUNNEL_PARAMETER_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.tunnelParameterError,
        parameterError: convertFromParameterError(state.parameterError),
      };
    case grpcTypes.ErrorState.Cause.SET_FIREWALL_POLICY_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.setFirewallPolicyError,
        policyError: convertFromBlockingError(state.policyError!),
      };

    case grpcTypes.ErrorState.Cause.IS_OFFLINE:
      return {
        ...baseError,
        cause: ErrorStateCause.isOffline,
      };
    case grpcTypes.ErrorState.Cause.SET_DNS_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.setDnsError,
      };
    case grpcTypes.ErrorState.Cause.IPV6_UNAVAILABLE:
      return {
        ...baseError,
        cause: ErrorStateCause.ipv6Unavailable,
      };
    case grpcTypes.ErrorState.Cause.START_TUNNEL_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.startTunnelError,
      };
    case grpcTypes.ErrorState.Cause.CREATE_TUNNEL_DEVICE:
      return {
        ...baseError,
        cause: ErrorStateCause.createTunnelDeviceError,
        osError: state.createTunnelError,
      };
    case grpcTypes.ErrorState.Cause.SPLIT_TUNNEL_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.splitTunnelError,
      };
    case grpcTypes.ErrorState.Cause.NEED_FULL_DISK_PERMISSIONS:
      return {
        ...baseError,
        cause: ErrorStateCause.needFullDiskPermissions,
      };
    // These are only ever created on Android
    case grpcTypes.ErrorState.Cause.INVALID_DNS_SERVERS:
    case grpcTypes.ErrorState.Cause.NOT_PREPARED:
    case grpcTypes.ErrorState.Cause.OTHER_ALWAYS_ON_APP:
    case grpcTypes.ErrorState.Cause.OTHER_LEGACY_ALWAYS_ON_VPN:
      throw new Error('Unsupported error state cause: ' + state.cause);
  }
}

function convertFromBlockingError(
  error: grpcTypes.ErrorState.FirewallPolicyError.AsObject,
): FirewallPolicyError {
  switch (error.type) {
    case grpcTypes.ErrorState.FirewallPolicyError.ErrorType.GENERIC:
      return { type: FirewallPolicyErrorType.generic };
    case grpcTypes.ErrorState.FirewallPolicyError.ErrorType.LOCKED: {
      const pid = error.lockPid;
      const name = error.lockName!;
      return { type: FirewallPolicyErrorType.locked, pid, name };
    }
  }
}

function convertFromAuthFailedError(error: grpcTypes.ErrorState.AuthFailedError): AuthFailedError {
  switch (error) {
    case grpcTypes.ErrorState.AuthFailedError.UNKNOWN:
      return AuthFailedError.unknown;
    case grpcTypes.ErrorState.AuthFailedError.INVALID_ACCOUNT:
      return AuthFailedError.invalidAccount;
    case grpcTypes.ErrorState.AuthFailedError.EXPIRED_ACCOUNT:
      return AuthFailedError.expiredAccount;
    case grpcTypes.ErrorState.AuthFailedError.TOO_MANY_CONNECTIONS:
      return AuthFailedError.tooManyConnections;
  }
}

function convertFromParameterError(
  error: grpcTypes.ErrorState.GenerationError,
): TunnelParameterError {
  switch (error) {
    case grpcTypes.ErrorState.GenerationError.NO_MATCHING_RELAY:
      return TunnelParameterError.noMatchingRelay;
    case grpcTypes.ErrorState.GenerationError.NO_MATCHING_BRIDGE_RELAY:
      return TunnelParameterError.noMatchingBridgeRelay;
    case grpcTypes.ErrorState.GenerationError.NO_WIREGUARD_KEY:
      return TunnelParameterError.noWireguardKey;
    case grpcTypes.ErrorState.GenerationError.CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR:
      return TunnelParameterError.customTunnelHostResolutionError;
    case grpcTypes.ErrorState.GenerationError.NETWORK_IPV4_UNAVAILABLE:
      return TunnelParameterError.ipv4Unavailable;
    case grpcTypes.ErrorState.GenerationError.NETWORK_IPV6_UNAVAILABLE:
      return TunnelParameterError.ipv6Unavailable;
  }
}

function convertFromTunnelStateRelayInfo(
  state: grpcTypes.TunnelStateRelayInfo.AsObject,
): ITunnelStateRelayInfo | undefined {
  if (state.tunnelEndpoint) {
    return {
      ...state,
      endpoint: {
        ...state.tunnelEndpoint,
        tunnelType: convertFromTunnelType(state.tunnelEndpoint.tunnelType),
        protocol: convertFromTransportProtocol(state.tunnelEndpoint.protocol),
        proxy: state.tunnelEndpoint.proxy && convertFromProxyEndpoint(state.tunnelEndpoint.proxy),
        obfuscationEndpoint:
          state.tunnelEndpoint.obfuscation &&
          convertFromObfuscationEndpoint(state.tunnelEndpoint.obfuscation),
        entryEndpoint:
          state.tunnelEndpoint.entryEndpoint &&
          convertFromEntryEndpoint(state.tunnelEndpoint.entryEndpoint),
      },
    };
  }
  return undefined;
}

function convertFromFeatureIndicators(
  featureIndicators?: Array<grpcTypes.FeatureIndicator>,
): Array<FeatureIndicator> | undefined {
  return featureIndicators?.map(convertFromFeatureIndicator);
}

function convertFromFeatureIndicator(
  featureIndicator: grpcTypes.FeatureIndicator,
): FeatureIndicator {
  switch (featureIndicator) {
    case grpcTypes.FeatureIndicator.QUANTUM_RESISTANCE:
      return FeatureIndicator.quantumResistance;
    case grpcTypes.FeatureIndicator.MULTIHOP:
      return FeatureIndicator.multihop;
    case grpcTypes.FeatureIndicator.BRIDGE_MODE:
      return FeatureIndicator.bridgeMode;
    case grpcTypes.FeatureIndicator.SPLIT_TUNNELING:
      return FeatureIndicator.splitTunneling;
    case grpcTypes.FeatureIndicator.LOCKDOWN_MODE:
      return FeatureIndicator.lockdownMode;
    case grpcTypes.FeatureIndicator.UDP_2_TCP:
      return FeatureIndicator.udp2tcp;
    case grpcTypes.FeatureIndicator.LAN_SHARING:
      return FeatureIndicator.lanSharing;
    case grpcTypes.FeatureIndicator.DNS_CONTENT_BLOCKERS:
      return FeatureIndicator.dnsContentBlockers;
    case grpcTypes.FeatureIndicator.CUSTOM_DNS:
      return FeatureIndicator.customDns;
    case grpcTypes.FeatureIndicator.SERVER_IP_OVERRIDE:
      return FeatureIndicator.serverIpOverride;
    case grpcTypes.FeatureIndicator.CUSTOM_MTU:
      return FeatureIndicator.customMtu;
    case grpcTypes.FeatureIndicator.CUSTOM_MSS_FIX:
      return FeatureIndicator.customMssFix;
    case grpcTypes.FeatureIndicator.DAITA:
      return FeatureIndicator.daita;
    case grpcTypes.FeatureIndicator.SHADOWSOCKS:
      return FeatureIndicator.shadowsocks;
  }
}

function convertFromTunnelType(tunnelType: grpcTypes.TunnelType): TunnelType {
  const tunnelTypeMap: Record<grpcTypes.TunnelType, TunnelType> = {
    [grpcTypes.TunnelType.WIREGUARD]: 'wireguard',
    [grpcTypes.TunnelType.OPENVPN]: 'openvpn',
  };

  return tunnelTypeMap[tunnelType];
}

function convertFromProxyEndpoint(proxyEndpoint: grpcTypes.ProxyEndpoint.AsObject): IProxyEndpoint {
  const proxyTypeMap: Record<grpcTypes.ProxyEndpoint.ProxyType, ProxyType> = {
    [grpcTypes.ProxyEndpoint.ProxyType.CUSTOM]: 'custom',
    [grpcTypes.ProxyEndpoint.ProxyType.SHADOWSOCKS]: 'shadowsocks',
  };

  return {
    ...proxyEndpoint,
    protocol: convertFromTransportProtocol(proxyEndpoint.protocol),
    proxyType: proxyTypeMap[proxyEndpoint.proxyType],
  };
}

function convertFromObfuscationEndpoint(
  obfuscationEndpoint: grpcTypes.ObfuscationEndpoint.AsObject,
): IObfuscationEndpoint {
  let obfuscationType: EndpointObfuscationType;
  switch (obfuscationEndpoint.obfuscationType) {
    case grpcTypes.ObfuscationEndpoint.ObfuscationType.UDP2TCP:
      obfuscationType = 'udp2tcp';
      break;
    case grpcTypes.ObfuscationEndpoint.ObfuscationType.SHADOWSOCKS:
      obfuscationType = 'shadowsocks';
      break;
    default:
      throw new Error('unsupported obfuscation protocol');
  }

  return {
    ...obfuscationEndpoint,
    protocol: convertFromTransportProtocol(obfuscationEndpoint.protocol),
    obfuscationType: obfuscationType,
  };
}

function convertFromEntryEndpoint(entryEndpoint: grpcTypes.Endpoint.AsObject) {
  return {
    address: entryEndpoint.address,
    transportProtocol: convertFromTransportProtocol(entryEndpoint.protocol),
  };
}

export function convertFromSettings(settings: grpcTypes.Settings): ISettings | undefined {
  const settingsObject = settings.toObject();
  const bridgeState = convertFromBridgeState(settingsObject.bridgeState!.state!);
  const relaySettings = convertFromRelaySettings(settings.getRelaySettings())!;
  const bridgeSettings = convertFromBridgeSettings(settings.getBridgeSettings()!);
  const tunnelOptions = convertFromTunnelOptions(settingsObject.tunnelOptions!);
  const splitTunnel = settingsObject.splitTunnel ?? { enableExclusions: false, appsList: [] };
  const obfuscationSettings = convertFromObfuscationSettings(settingsObject.obfuscationSettings);
  const customLists = convertFromCustomListSettings(settings.getCustomLists());
  const apiAccessMethods = convertFromApiAccessMethodSettings(settings.getApiAccessMethods()!);
  const relayOverrides = settingsObject.relayOverridesList;
  return {
    ...settings.toObject(),
    bridgeState,
    relaySettings,
    bridgeSettings,
    tunnelOptions,
    splitTunnel,
    obfuscationSettings,
    customLists,
    apiAccessMethods,
    relayOverrides,
  };
}

function convertFromBridgeState(bridgeState: grpcTypes.BridgeState.State): BridgeState {
  const bridgeStateMap: Record<grpcTypes.BridgeState.State, BridgeState> = {
    [grpcTypes.BridgeState.State.AUTO]: 'auto',
    [grpcTypes.BridgeState.State.ON]: 'on',
    [grpcTypes.BridgeState.State.OFF]: 'off',
  };

  return bridgeStateMap[bridgeState];
}

function convertFromRelaySettings(
  relaySettings?: grpcTypes.RelaySettings,
): RelaySettings | undefined {
  if (relaySettings) {
    switch (relaySettings.getEndpointCase()) {
      case grpcTypes.RelaySettings.EndpointCase.ENDPOINT_NOT_SET:
        return undefined;
      case grpcTypes.RelaySettings.EndpointCase.CUSTOM: {
        const custom = relaySettings.getCustom()?.toObject();
        const config = relaySettings.getCustom()?.getConfig();
        const connectionConfig = config && convertFromConnectionConfig(config);
        return (
          custom &&
          connectionConfig && {
            customTunnelEndpoint: {
              ...custom,
              config: connectionConfig,
            },
          }
        );
      }
      case grpcTypes.RelaySettings.EndpointCase.NORMAL: {
        const normal = relaySettings.getNormal()!;
        const locationConstraint = convertFromLocationConstraint(normal.getLocation());
        const location = wrapConstraint(locationConstraint);
        const tunnelProtocol = convertFromTunnelType(normal.getTunnelType());
        const providers = normal.getProvidersList();
        const ownership = convertFromOwnership(normal.getOwnership());
        const openvpnConstraints = convertFromOpenVpnConstraints(normal.getOpenvpnConstraints()!);
        const wireguardConstraints = convertFromWireguardConstraints(
          normal.getWireguardConstraints()!,
        );

        return {
          normal: {
            location,
            tunnelProtocol,
            providers,
            ownership,
            wireguardConstraints,
            openvpnConstraints,
          },
        };
      }
    }
  } else {
    return undefined;
  }
}

function convertFromBridgeSettings(bridgeSettings: grpcTypes.BridgeSettings): BridgeSettings {
  const bridgeSettingsObject = bridgeSettings.toObject();

  const detailsMap: Record<grpcTypes.BridgeSettings.BridgeType, BridgeType> = {
    [grpcTypes.BridgeSettings.BridgeType.NORMAL]: 'normal',
    [grpcTypes.BridgeSettings.BridgeType.CUSTOM]: 'custom',
  };
  const type = detailsMap[bridgeSettingsObject.bridgeType];

  const normalSettings = bridgeSettingsObject.normal;
  const locationConstraint = convertFromLocationConstraint(
    bridgeSettings.getNormal()?.getLocation(),
  );
  const location = wrapConstraint(locationConstraint);
  const providers = normalSettings!.providersList;
  const ownership = convertFromOwnership(normalSettings!.ownership);

  const normal = {
    location,
    providers,
    ownership,
  };

  const grpcCustom = bridgeSettings.getCustom();
  const custom = grpcCustom ? convertFromCustomProxy(grpcCustom) : undefined;

  return { type, normal, custom };
}

function convertFromConnectionConfig(
  connectionConfig: grpcTypes.ConnectionConfig,
): ConnectionConfig | undefined {
  const connectionConfigObject = connectionConfig.toObject();
  switch (connectionConfig.getConfigCase()) {
    case grpcTypes.ConnectionConfig.ConfigCase.CONFIG_NOT_SET:
      return undefined;
    case grpcTypes.ConnectionConfig.ConfigCase.WIREGUARD:
      return (
        connectionConfigObject.wireguard &&
        connectionConfigObject.wireguard.tunnel &&
        connectionConfigObject.wireguard.peer && {
          wireguard: {
            ...connectionConfigObject.wireguard,
            tunnel: {
              privateKey: convertFromWireguardKey(
                connectionConfigObject.wireguard.tunnel.privateKey,
              ),
              addresses: connectionConfigObject.wireguard.tunnel.addressesList,
            },
            peer: {
              ...connectionConfigObject.wireguard.peer,
              addresses: connectionConfigObject.wireguard.peer.allowedIpsList,
              publicKey: convertFromWireguardKey(connectionConfigObject.wireguard.peer.publicKey),
            },
          },
        }
      );
    case grpcTypes.ConnectionConfig.ConfigCase.OPENVPN: {
      const [ip, port] = connectionConfigObject.openvpn!.address.split(':');
      return {
        openvpn: {
          ...connectionConfigObject.openvpn!,
          endpoint: {
            ip,
            port: parseInt(port, 10),
            protocol: convertFromTransportProtocol(connectionConfigObject.openvpn!.protocol),
          },
        },
      };
    }
  }
}

function convertFromLocationConstraint(
  location?: grpcTypes.LocationConstraint,
): RelayLocation | undefined {
  if (location === undefined) {
    return undefined;
  } else if (location.getTypeCase() === grpcTypes.LocationConstraint.TypeCase.CUSTOM_LIST) {
    return { customList: location.getCustomList() };
  } else {
    const innerLocation = location.getLocation()?.toObject();
    return innerLocation && convertFromGeographicConstraint(innerLocation);
  }
}

function convertFromGeographicConstraint(
  location: grpcTypes.GeographicLocationConstraint.AsObject,
): RelayLocation {
  if (location.hostname) {
    return location;
  } else if (location.city) {
    return {
      country: location.country,
      city: location.city,
    };
  } else {
    return {
      country: location.country,
    };
  }
}

function convertFromTunnelOptions(tunnelOptions: grpcTypes.TunnelOptions.AsObject): ITunnelOptions {
  return {
    openvpn: {
      mssfix: tunnelOptions.openvpn!.mssfix,
    },
    wireguard: {
      mtu: tunnelOptions.wireguard!.mtu,
      quantumResistant: convertFromQuantumResistantState(
        tunnelOptions.wireguard?.quantumResistant?.state,
      ),
      daita: tunnelOptions.wireguard!.daita,
    },
    generic: {
      enableIpv6: tunnelOptions.generic!.enableIpv6,
    },
    dns: {
      state:
        tunnelOptions.dnsOptions?.state === grpcTypes.DnsOptions.DnsState.CUSTOM
          ? 'custom'
          : 'default',
      defaultOptions: {
        blockAds: tunnelOptions.dnsOptions?.defaultOptions?.blockAds ?? false,
        blockTrackers: tunnelOptions.dnsOptions?.defaultOptions?.blockTrackers ?? false,
        blockMalware: tunnelOptions.dnsOptions?.defaultOptions?.blockMalware ?? false,
        blockAdultContent: tunnelOptions.dnsOptions?.defaultOptions?.blockAdultContent ?? false,
        blockGambling: tunnelOptions.dnsOptions?.defaultOptions?.blockGambling ?? false,
        blockSocialMedia: tunnelOptions.dnsOptions?.defaultOptions?.blockSocialMedia ?? false,
      },
      customOptions: {
        addresses: tunnelOptions.dnsOptions?.customOptions?.addressesList ?? [],
      },
    },
  };
}

function convertFromQuantumResistantState(
  state?: grpcTypes.QuantumResistantState.State,
): boolean | undefined {
  return state === undefined
    ? undefined
    : {
        [grpcTypes.QuantumResistantState.State.ON]: true,
        [grpcTypes.QuantumResistantState.State.OFF]: false,
        [grpcTypes.QuantumResistantState.State.AUTO]: undefined,
      }[state];
}

function convertFromObfuscationSettings(
  obfuscationSettings?: grpcTypes.ObfuscationSettings.AsObject,
): ObfuscationSettings {
  let selectedObfuscationType = ObfuscationType.auto;
  switch (obfuscationSettings?.selectedObfuscation) {
    case grpcTypes.ObfuscationSettings.SelectedObfuscation.OFF:
      selectedObfuscationType = ObfuscationType.off;
      break;
    case grpcTypes.ObfuscationSettings.SelectedObfuscation.UDP2TCP:
      selectedObfuscationType = ObfuscationType.udp2tcp;
      break;
    case grpcTypes.ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS:
      selectedObfuscationType = ObfuscationType.shadowsocks;
      break;
  }

  return {
    selectedObfuscation: selectedObfuscationType,
    udp2tcpSettings: obfuscationSettings?.udp2tcp
      ? { port: convertFromConstraint(obfuscationSettings.udp2tcp.port) }
      : { port: 'any' },
    shadowsocksSettings: obfuscationSettings?.shadowsocks
      ? { port: convertFromConstraint(obfuscationSettings.shadowsocks.port) }
      : { port: 'any' },
  };
}

export function convertFromDaemonEvent(data: grpcTypes.DaemonEvent): DaemonEvent {
  const tunnelState = data.getTunnelState();
  if (tunnelState !== undefined) {
    return { tunnelState: convertFromTunnelState(tunnelState)! };
  }

  const settings = data.getSettings();
  if (settings !== undefined) {
    return { settings: convertFromSettings(settings)! };
  }

  const relayList = data.getRelayList();
  if (relayList !== undefined) {
    return { relayList: convertFromRelayList(relayList) };
  }

  const deviceConfig = data.getDevice();
  if (deviceConfig !== undefined) {
    return { device: convertFromDeviceEvent(deviceConfig) };
  }

  const deviceRemoval = data.getRemoveDevice();
  if (deviceRemoval !== undefined) {
    return { deviceRemoval: convertFromDeviceRemoval(deviceRemoval) };
  }

  const versionInfo = data.getVersionInfo();
  if (versionInfo !== undefined) {
    return { appVersionInfo: versionInfo.toObject() };
  }

  const newAccessMethod = data.getNewAccessMethod();
  if (newAccessMethod !== undefined) {
    return { accessMethodSetting: convertFromApiAccessMethodSetting(newAccessMethod) };
  }

  // Handle unknown daemon events
  const keys = Object.entries(data.toObject())
    .filter(([, value]) => value !== undefined)
    .map(([key]) => key);
  throw new Error(`Unknown daemon event received containing ${keys}`);
}

function convertFromOwnership(ownership: grpcTypes.Ownership): Ownership {
  switch (ownership) {
    case grpcTypes.Ownership.ANY:
      return Ownership.any;
    case grpcTypes.Ownership.MULLVAD_OWNED:
      return Ownership.mullvadOwned;
    case grpcTypes.Ownership.RENTED:
      return Ownership.rented;
  }
}

function convertToOwnership(ownership: Ownership): grpcTypes.Ownership {
  switch (ownership) {
    case Ownership.any:
      return grpcTypes.Ownership.ANY;
    case Ownership.mullvadOwned:
      return grpcTypes.Ownership.MULLVAD_OWNED;
    case Ownership.rented:
      return grpcTypes.Ownership.RENTED;
  }
}

function convertFromOpenVpnConstraints(
  constraints: grpcTypes.OpenvpnConstraints,
): IOpenVpnConstraints {
  const transportPort = convertFromConstraint(constraints.getPort());
  if (transportPort !== 'any' && 'only' in transportPort) {
    const port = convertFromConstraint(transportPort.only.getPort());
    let protocol: Constraint<RelayProtocol> = 'any';
    switch (transportPort.only.getProtocol()) {
      case grpcTypes.TransportProtocol.TCP:
        protocol = { only: 'tcp' };
        break;
      case grpcTypes.TransportProtocol.UDP:
        protocol = { only: 'udp' };
        break;
    }
    return { port, protocol };
  }
  return { port: 'any', protocol: 'any' };
}

function convertFromWireguardConstraints(
  constraints: grpcTypes.WireguardConstraints,
): IWireguardConstraints {
  const result: IWireguardConstraints = {
    port: 'any',
    ipVersion: 'any',
    useMultihop: constraints.getUseMultihop(),
    entryLocation: 'any',
  };

  const port = constraints.getPort();
  if (port) {
    result.port = { only: port };
  }

  // `getIpVersion()` is not falsy if type is 'any'
  if (constraints.hasIpVersion()) {
    switch (constraints.getIpVersion()) {
      case grpcTypes.IpVersion.V4:
        result.ipVersion = { only: 'ipv4' };
        break;
      case grpcTypes.IpVersion.V6:
        result.ipVersion = { only: 'ipv6' };
        break;
    }
  }

  const entryLocation = constraints.getEntryLocation();
  if (entryLocation) {
    const location = convertFromLocationConstraint(entryLocation);
    result.entryLocation = wrapConstraint(location);
  }

  return result;
}

function convertFromConstraint<T>(value: T | undefined): Constraint<T> {
  if (value) {
    return { only: value };
  } else {
    return 'any';
  }
}

export function convertToRelayConstraints(
  constraints: IRelaySettingsNormal<IOpenVpnConstraints, IWireguardConstraints>,
): grpcTypes.NormalRelaySettings {
  const relayConstraints = new grpcTypes.NormalRelaySettings();

  relayConstraints.setTunnelType(convertToTunnelType(constraints.tunnelProtocol));
  relayConstraints.setLocation(convertToLocation(unwrapConstraint(constraints.location)));
  relayConstraints.setWireguardConstraints(
    convertToWireguardConstraints(constraints.wireguardConstraints),
  );
  relayConstraints.setOpenvpnConstraints(
    convertToOpenVpnConstraints(constraints.openvpnConstraints),
  );
  relayConstraints.setProvidersList(constraints.providers);
  relayConstraints.setOwnership(convertToOwnership(constraints.ownership));

  return relayConstraints;
}

export function convertToNormalBridgeSettings(
  constraints: IBridgeConstraints,
): grpcTypes.BridgeSettings.BridgeConstraints {
  const normalBridgeSettings = new grpcTypes.BridgeSettings.BridgeConstraints();
  normalBridgeSettings.setLocation(convertToLocation(unwrapConstraint(constraints.location)));
  normalBridgeSettings.setProvidersList(constraints.providers);

  return normalBridgeSettings;
}

function convertToLocation(
  constraint: RelayLocation | undefined,
): grpcTypes.LocationConstraint | undefined {
  const locationConstraint = new grpcTypes.LocationConstraint();
  if (constraint && 'customList' in constraint && constraint.customList) {
    locationConstraint.setCustomList(constraint.customList);
  } else {
    const location = constraint && convertToGeographicConstraint(constraint);
    locationConstraint.setLocation(location);
  }

  return locationConstraint;
}

function convertToGeographicConstraint(
  location: RelayLocation,
): grpcTypes.GeographicLocationConstraint {
  const relayLocation = new grpcTypes.GeographicLocationConstraint();
  if ('hostname' in location) {
    relayLocation.setCountry(location.country);
    relayLocation.setCity(location.city);
    relayLocation.setHostname(location.hostname);
  } else if ('city' in location) {
    relayLocation.setCountry(location.country);
    relayLocation.setCity(location.city);
  } else if ('country' in location) {
    relayLocation.setCountry(location.country);
  }

  return relayLocation;
}

function convertToTunnelType(tunnelProtocol: TunnelProtocol): grpcTypes.TunnelType {
  switch (tunnelProtocol) {
    case 'wireguard':
      return grpcTypes.TunnelType.WIREGUARD;
    case 'openvpn':
      return grpcTypes.TunnelType.OPENVPN;
  }
}

function convertToOpenVpnConstraints(
  constraints: Partial<IOpenVpnConstraints> | undefined,
): grpcTypes.OpenvpnConstraints | undefined {
  const openvpnConstraints = new grpcTypes.OpenvpnConstraints();
  if (constraints) {
    const protocol = unwrapConstraint(constraints.protocol);
    if (protocol) {
      const portConstraints = new grpcTypes.TransportPort();
      const port = unwrapConstraint(constraints.port);
      if (port) {
        portConstraints.setPort(port);
      }
      portConstraints.setProtocol(convertToTransportProtocol(protocol));
      openvpnConstraints.setPort(portConstraints);
    }
    return openvpnConstraints;
  }

  return undefined;
}

function convertToWireguardConstraints(
  constraint: Partial<IWireguardConstraints> | undefined,
): grpcTypes.WireguardConstraints | undefined {
  if (constraint) {
    const wireguardConstraints = new grpcTypes.WireguardConstraints();

    const port = unwrapConstraint(constraint.port);
    if (port) {
      wireguardConstraints.setPort(port);
    }

    const ipVersion = unwrapConstraint(constraint.ipVersion);
    if (ipVersion) {
      const ipVersionProtocol =
        ipVersion === 'ipv4' ? grpcTypes.IpVersion.V4 : grpcTypes.IpVersion.V6;
      wireguardConstraints.setIpVersion(ipVersionProtocol);
    }

    if (constraint.useMultihop) {
      wireguardConstraints.setUseMultihop(constraint.useMultihop);
    }

    const entryLocation = unwrapConstraint(constraint.entryLocation);
    if (entryLocation) {
      const entryLocationConstraint = convertToLocation(entryLocation);
      wireguardConstraints.setEntryLocation(entryLocationConstraint);
    }

    return wireguardConstraints;
  }
  return undefined;
}

function convertToTransportProtocol(protocol: RelayProtocol): grpcTypes.TransportProtocol {
  switch (protocol) {
    case 'udp':
      return grpcTypes.TransportProtocol.UDP;
    case 'tcp':
      return grpcTypes.TransportProtocol.TCP;
  }
}

function convertFromDeviceEvent(deviceEvent: grpcTypes.DeviceEvent): DeviceEvent {
  const deviceState = convertFromDeviceState(deviceEvent.getNewState()!);
  switch (deviceEvent.getCause()) {
    case grpcTypes.DeviceEvent.Cause.LOGGED_IN:
      return { type: 'logged in', deviceState: deviceState as LoggedInDeviceState };
    case grpcTypes.DeviceEvent.Cause.LOGGED_OUT:
      return { type: 'logged out', deviceState: deviceState as LoggedOutDeviceState };
    case grpcTypes.DeviceEvent.Cause.REVOKED:
      return { type: 'revoked', deviceState: deviceState as LoggedOutDeviceState };
    case grpcTypes.DeviceEvent.Cause.UPDATED:
      return { type: 'updated', deviceState: deviceState as LoggedInDeviceState };
    case grpcTypes.DeviceEvent.Cause.ROTATED_KEY:
      return { type: 'rotated_key', deviceState: deviceState as LoggedInDeviceState };
  }
}

export function convertFromDeviceState(deviceState: grpcTypes.DeviceState): DeviceState {
  switch (deviceState.getState()) {
    case grpcTypes.DeviceState.State.LOGGED_IN: {
      const accountAndDevice = deviceState.getDevice()!;
      const device = accountAndDevice.getDevice();
      return {
        type: 'logged in',
        accountAndDevice: {
          accountNumber: accountAndDevice.getAccountNumber(),
          device: device && convertFromDevice(device),
        },
      };
    }
    case grpcTypes.DeviceState.State.LOGGED_OUT:
      return { type: 'logged out' };
    case grpcTypes.DeviceState.State.REVOKED:
      return { type: 'revoked' };
  }
}

function convertFromDeviceRemoval(deviceRemoval: grpcTypes.RemoveDeviceEvent): Array<IDevice> {
  return deviceRemoval.getNewDeviceListList().map(convertFromDevice);
}

export function convertFromDevice(device: grpcTypes.Device): IDevice {
  const created = ensureExists(device.getCreated(), "no 'created' field for device").toDate();
  const asObject = device.toObject();

  return {
    ...asObject,
    created: created,
  };
}

function convertFromCustomListSettings(
  customListSettings?: grpcTypes.CustomListSettings,
): CustomLists {
  return customListSettings ? convertFromCustomLists(customListSettings.getCustomListsList()) : [];
}

function convertFromCustomLists(customLists: Array<grpcTypes.CustomList>): CustomLists {
  return customLists.map((list) => ({
    id: list.getId(),
    name: list.getName(),
    locations: list
      .getLocationsList()
      .map((location) =>
        convertFromGeographicConstraint(location.toObject()),
      ) as Array<RelayLocationGeographical>,
  }));
}

export function convertToCustomList(customList: ICustomList): grpcTypes.CustomList {
  const grpcCustomList = new grpcTypes.CustomList();
  grpcCustomList.setId(customList.id);
  grpcCustomList.setName(customList.name);

  const locations = customList.locations.map(convertToGeographicConstraint);
  grpcCustomList.setLocationsList(locations);

  return grpcCustomList;
}

export function convertToApiAccessMethodSetting(
  method: AccessMethodSetting,
): grpcTypes.AccessMethodSetting {
  const updatedMethod = new grpcTypes.AccessMethodSetting();
  const uuid = new grpcTypes.UUID();
  uuid.setValue(method.id);
  updatedMethod.setId(uuid);
  return fillApiAccessMethodSetting(updatedMethod, method);
}

export function convertToNewApiAccessMethodSetting(
  method: NewAccessMethodSetting,
): grpcTypes.NewAccessMethodSetting {
  const newMethod = new grpcTypes.NewAccessMethodSetting();
  return fillApiAccessMethodSetting(newMethod, method);
}

function fillApiAccessMethodSetting<T extends grpcTypes.NewAccessMethodSetting>(
  newMethod: T,
  method: NewAccessMethodSetting,
): T {
  newMethod.setName(method.name);
  newMethod.setEnabled(method.enabled);

  const accessMethod = new grpcTypes.AccessMethod();
  switch (method.type) {
    case 'direct': {
      const direct = new grpcTypes.AccessMethod.Direct();
      accessMethod.setDirect(direct);
      break;
    }
    case 'bridges': {
      const bridges = new grpcTypes.AccessMethod.Bridges();
      accessMethod.setBridges(bridges);
      break;
    }
    case 'encrypted-dns-proxy': {
      const encryptedDnsProxy = new grpcTypes.AccessMethod.EncryptedDnsProxy();
      accessMethod.setEncryptedDnsProxy(encryptedDnsProxy);
      break;
    }
    default:
      accessMethod.setCustom(convertToCustomProxy(method));
  }

  newMethod.setAccessMethod(accessMethod);
  return newMethod;
}

export function convertToCustomProxy(proxy: CustomProxy): grpcTypes.CustomProxy {
  const customProxy = new grpcTypes.CustomProxy();

  switch (proxy.type) {
    case 'socks5-local': {
      const socks5Local = new grpcTypes.Socks5Local();
      socks5Local.setRemoteIp(proxy.remoteIp);
      socks5Local.setRemotePort(proxy.remotePort);
      socks5Local.setRemoteTransportProtocol(
        convertToTransportProtocol(proxy.remoteTransportProtocol),
      );
      socks5Local.setLocalPort(proxy.localPort);
      customProxy.setSocks5local(socks5Local);
      break;
    }
    case 'socks5-remote': {
      const socks5Remote = new grpcTypes.Socks5Remote();
      socks5Remote.setIp(proxy.ip);
      socks5Remote.setPort(proxy.port);
      if (proxy.authentication !== undefined) {
        socks5Remote.setAuth(convertToSocksAuth(proxy.authentication));
      }
      customProxy.setSocks5remote(socks5Remote);
      break;
    }
    case 'shadowsocks': {
      const shadowsocks = new grpcTypes.Shadowsocks();
      shadowsocks.setIp(proxy.ip);
      shadowsocks.setPort(proxy.port);
      shadowsocks.setPassword(proxy.password);
      shadowsocks.setCipher(proxy.cipher);
      customProxy.setShadowsocks(shadowsocks);
      break;
    }
  }

  return customProxy;
}

function convertToSocksAuth(authentication: SocksAuth): grpcTypes.SocksAuth {
  const auth = new grpcTypes.SocksAuth();
  auth.setUsername(authentication.username);
  auth.setPassword(authentication.password);
  return auth;
}

function convertFromApiAccessMethodSettings(
  accessMethods: grpcTypes.ApiAccessMethodSettings,
): ApiAccessMethodSettings {
  const direct = convertFromApiAccessMethodSetting(
    ensureExists(accessMethods.getDirect(), "no 'Direct' access method was found"),
  ) as AccessMethodSetting<DirectMethod>;
  const bridges = convertFromApiAccessMethodSetting(
    ensureExists(accessMethods.getMullvadBridges(), "no 'Mullvad Bridges' access method was found"),
  ) as AccessMethodSetting<BridgesMethod>;
  const encryptedDnsProxy = convertFromApiAccessMethodSetting(
    ensureExists(
      accessMethods.getEncryptedDnsProxy(),
      "no 'Encrypted DNS proxy' access method was found",
    ),
  ) as AccessMethodSetting<EncryptedDnsProxy>;
  const custom = accessMethods
    .getCustomList()
    .filter((setting) => setting.hasId() && setting.hasAccessMethod())
    .map(convertFromApiAccessMethodSetting)
    // The last filter helps TypeScript infer the custom proxy type.
    .filter(isCustomProxy);

  return {
    direct,
    mullvadBridges: bridges,
    encryptedDnsProxy,
    custom,
  };
}

function isCustomProxy(
  accessMethod: AccessMethodSetting,
): accessMethod is AccessMethodSetting<CustomProxy> {
  return (
    accessMethod.type !== 'direct' &&
    accessMethod.type !== 'bridges' &&
    accessMethod.type !== 'encrypted-dns-proxy'
  );
}

export function convertFromApiAccessMethodSetting(
  setting: grpcTypes.AccessMethodSetting,
): AccessMethodSetting {
  const id = setting.getId()!;
  const accessMethod = setting.getAccessMethod()!;

  return {
    id: id.getValue(),
    name: setting.getName(),
    enabled: setting.getEnabled(),
    ...convertFromAccessMethod(accessMethod),
  };
}

function convertFromAccessMethod(method: grpcTypes.AccessMethod): AccessMethod {
  switch (method.getAccessMethodCase()) {
    case grpcTypes.AccessMethod.AccessMethodCase.DIRECT:
      return { type: 'direct' };
    case grpcTypes.AccessMethod.AccessMethodCase.BRIDGES:
      return { type: 'bridges' };
    case grpcTypes.AccessMethod.AccessMethodCase.ENCRYPTED_DNS_PROXY:
      return { type: 'encrypted-dns-proxy' };
    case grpcTypes.AccessMethod.AccessMethodCase.CUSTOM: {
      return convertFromCustomProxy(method.getCustom()!);
    }
    case grpcTypes.AccessMethod.AccessMethodCase.ACCESS_METHOD_NOT_SET:
      throw new Error('Access method not set, which should always be set');
  }
}

function convertFromCustomProxy(proxy: grpcTypes.CustomProxy): CustomProxy {
  switch (proxy.getProxyMethodCase()) {
    case grpcTypes.CustomProxy.ProxyMethodCase.SOCKS5LOCAL: {
      const socks5Local = proxy.getSocks5local()!;
      return {
        type: 'socks5-local',
        remoteIp: socks5Local.getRemoteIp(),
        remotePort: socks5Local.getRemotePort(),
        remoteTransportProtocol: convertFromTransportProtocol(
          socks5Local.getRemoteTransportProtocol(),
        ),
        localPort: socks5Local.getLocalPort(),
      };
    }
    case grpcTypes.CustomProxy.ProxyMethodCase.SOCKS5REMOTE: {
      const socks5Remote = proxy.getSocks5remote()!;
      const auth = socks5Remote.getAuth();
      return {
        type: 'socks5-remote',
        ip: socks5Remote.getIp(),
        port: socks5Remote.getPort(),
        authentication: auth === undefined ? undefined : convertFromSocksAuth(auth),
      };
    }
    case grpcTypes.CustomProxy.ProxyMethodCase.SHADOWSOCKS: {
      const shadowsocks = proxy.getShadowsocks()!;
      return {
        type: 'shadowsocks',
        ip: shadowsocks.getIp(),
        port: shadowsocks.getPort(),
        password: shadowsocks.getPassword(),
        cipher: shadowsocks.getCipher(),
      };
    }
    case grpcTypes.CustomProxy.ProxyMethodCase.PROXY_METHOD_NOT_SET:
      throw new Error('Custom method not set, which should always be set');
  }
}

function convertFromSocksAuth(auth: grpcTypes.SocksAuth): SocksAuth {
  return {
    username: auth.getUsername(),
    password: auth.getPassword(),
  };
}

export function ensureExists<T>(value: T | undefined, errorMessage: string): T {
  if (value) {
    return value;
  }
  throw new ResponseParseError(errorMessage);
}
