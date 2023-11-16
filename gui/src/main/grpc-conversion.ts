import {
  AfterDisconnect,
  AuthFailedError,
  BridgeSettings,
  BridgeState,
  ConnectionConfig,
  Constraint,
  CustomLists,
  DaemonEvent,
  DeviceEvent,
  DeviceState,
  EndpointObfuscationType,
  ErrorState,
  ErrorStateCause,
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
  ObfuscationSettings,
  ObfuscationType,
  Ownership,
  ProxySettings,
  ProxyType,
  RelayEndpointType,
  RelayLocation,
  RelayLocationGeographical,
  RelayProtocol,
  RelaySettings,
  TunnelParameterError,
  TunnelProtocol,
  TunnelState,
  TunnelType,
  wrapConstraint,
} from '../shared/daemon-rpc-types';
import * as grpcTypes from './management_interface/management_interface_pb';

const invalidErrorStateCause = new Error(
  'VPN_PERMISSION_DENIED is not a valid error state cause on desktop',
);

export function liftConstraint<T>(constraint: Constraint<T> | undefined): T | undefined {
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
        .map((country: grpcTypes.RelayListCountry) =>
          convertFromRelayListCountry(country.toObject()),
        ),
    },
    wireguardEndpointData: convertWireguardEndpointData(relayList.getWireguard()!),
  };
}

export function convertWireguardEndpointData(
  data: grpcTypes.WireguardEndpointData,
): IWireguardEndpointData {
  return {
    portRanges: data.getPortRangesList().map((range) => [range.getFirst(), range.getLast()]),
    udp2tcpPorts: data.getUdp2tcpPortsList(),
  };
}

export function convertFromRelayListCountry(
  country: grpcTypes.RelayListCountry.AsObject,
): IRelayListCountry {
  return {
    ...country,
    cities: country.citiesList.map(convertFromRelayListCity),
  };
}

export function convertFromRelayListCity(city: grpcTypes.RelayListCity.AsObject): IRelayListCity {
  return {
    ...city,
    relays: city.relaysList.map(convertFromRelayListRelay),
  };
}

export function convertFromRelayListRelay(relay: grpcTypes.Relay.AsObject): IRelayListHostname {
  return {
    ...relay,
    endpointType: convertFromRelayType(relay.endpointType),
  };
}

export function convertFromRelayType(relayType: grpcTypes.Relay.RelayType): RelayEndpointType {
  const protocolMap: Record<grpcTypes.Relay.RelayType, RelayEndpointType> = {
    [grpcTypes.Relay.RelayType.OPENVPN]: 'openvpn',
    [grpcTypes.Relay.RelayType.BRIDGE]: 'bridge',
    [grpcTypes.Relay.RelayType.WIREGUARD]: 'wireguard',
  };
  return protocolMap[relayType];
}

export function convertFromWireguardKey(publicKey: Uint8Array | string): string {
  if (typeof publicKey === 'string') {
    return publicKey;
  }
  return Buffer.from(publicKey).toString('base64');
}

export function convertFromTransportProtocol(protocol: grpcTypes.TransportProtocol): RelayProtocol {
  const protocolMap: Record<grpcTypes.TransportProtocol, RelayProtocol> = {
    [grpcTypes.TransportProtocol.TCP]: 'tcp',
    [grpcTypes.TransportProtocol.UDP]: 'udp',
  };
  return protocolMap[protocol];
}

export function convertFromTunnelState(tunnelState: grpcTypes.TunnelState): TunnelState | undefined {
  const tunnelStateObject = tunnelState.toObject();
  switch (tunnelState.getStateCase()) {
    case grpcTypes.TunnelState.StateCase.STATE_NOT_SET:
      return undefined;
    case grpcTypes.TunnelState.StateCase.DISCONNECTED:
      return { state: 'disconnected' };
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
      };
    case grpcTypes.TunnelState.StateCase.CONNECTED: {
      const relayInfo =
        tunnelStateObject.connected?.relayInfo &&
        convertFromTunnelStateRelayInfo(tunnelStateObject.connected.relayInfo);
      return (
        relayInfo && {
          state: 'connected',
          details: relayInfo,
        }
      );
    }
  }
}

export function convertFromTunnelStateError(state: grpcTypes.ErrorState.AsObject): ErrorState {
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
    case grpcTypes.ErrorState.Cause.SPLIT_TUNNEL_ERROR:
      return {
        ...baseError,
        cause: ErrorStateCause.splitTunnelError,
      };
    case grpcTypes.ErrorState.Cause.VPN_PERMISSION_DENIED:
      // VPN_PERMISSION_DENIED is only ever created on Android
      throw invalidErrorStateCause;
  }
}

export function convertFromBlockingError(
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

export function convertFromAuthFailedError(error: grpcTypes.ErrorState.AuthFailedError): AuthFailedError {
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

export function convertFromParameterError(
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
  }
}

export function convertFromTunnelStateRelayInfo(
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

export function convertFromTunnelType(tunnelType: grpcTypes.TunnelType): TunnelType {
  const tunnelTypeMap: Record<grpcTypes.TunnelType, TunnelType> = {
    [grpcTypes.TunnelType.WIREGUARD]: 'wireguard',
    [grpcTypes.TunnelType.OPENVPN]: 'openvpn',
  };

  return tunnelTypeMap[tunnelType];
}

export function convertFromProxyEndpoint(proxyEndpoint: grpcTypes.ProxyEndpoint.AsObject): IProxyEndpoint {
  const proxyTypeMap: Record<grpcTypes.ProxyType, ProxyType> = {
    [grpcTypes.ProxyType.CUSTOM]: 'custom',
    [grpcTypes.ProxyType.SHADOWSOCKS]: 'shadowsocks',
  };

  return {
    ...proxyEndpoint,
    protocol: convertFromTransportProtocol(proxyEndpoint.protocol),
    proxyType: proxyTypeMap[proxyEndpoint.proxyType],
  };
}

export function convertFromObfuscationEndpoint(
  obfuscationEndpoint: grpcTypes.ObfuscationEndpoint.AsObject,
): IObfuscationEndpoint {
  const obfuscationTypes: Record<grpcTypes.ObfuscationType, EndpointObfuscationType> = {
    [grpcTypes.ObfuscationType.UDP2TCP]: 'udp2tcp',
  };

  return {
    ...obfuscationEndpoint,
    protocol: convertFromTransportProtocol(obfuscationEndpoint.protocol),
    obfuscationType: obfuscationTypes[obfuscationEndpoint.obfuscationType],
  };
}

export function convertFromEntryEndpoint(entryEndpoint: grpcTypes.Endpoint.AsObject) {
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
  return {
    ...settings.toObject(),
    bridgeState,
    relaySettings,
    bridgeSettings,
    tunnelOptions,
    splitTunnel,
    obfuscationSettings,
    customLists,
  };
}

export function convertFromBridgeState(bridgeState: grpcTypes.BridgeState.State): BridgeState {
  const bridgeStateMap: Record<grpcTypes.BridgeState.State, BridgeState> = {
    [grpcTypes.BridgeState.State.AUTO]: 'auto',
    [grpcTypes.BridgeState.State.ON]: 'on',
    [grpcTypes.BridgeState.State.OFF]: 'off',
  };

  return bridgeStateMap[bridgeState];
}

export function convertFromRelaySettings(
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
        // `getTunnelType()` is not falsy if type is 'any'
        const tunnelProtocol = convertFromTunnelTypeConstraint(
          normal.hasTunnelType() ? normal.getTunnelType() : undefined,
        );
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

export function convertFromBridgeSettings(bridgeSettings: grpcTypes.BridgeSettings): BridgeSettings {
  const bridgeSettingsObject = bridgeSettings.toObject();
  const normalSettings = bridgeSettingsObject.normal;
  if (normalSettings) {
    const locationConstraint = convertFromLocationConstraint(
      bridgeSettings.getNormal()?.getLocation(),
    );
    const location = wrapConstraint(locationConstraint);
    const providers = normalSettings.providersList;
    const ownership = convertFromOwnership(normalSettings.ownership);
    return {
      normal: {
        location,
        providers,
        ownership,
      },
    };
  }

  const customSettings = (settings: ProxySettings): BridgeSettings => {
    return { custom: settings };
  };

  const localSettings = bridgeSettingsObject.local;
  if (localSettings) {
    return customSettings({
      port: localSettings.port,
      peer: localSettings.peer,
    });
  }

  const remoteSettings = bridgeSettingsObject.remote;
  if (remoteSettings) {
    return customSettings({
      address: remoteSettings.address,
      auth: remoteSettings.auth && { ...remoteSettings.auth },
    });
  }

  const shadowsocksSettings = bridgeSettingsObject.shadowsocks!;
  return customSettings({
    peer: shadowsocksSettings.peer!,
    password: shadowsocksSettings.password!,
    cipher: shadowsocksSettings.cipher!,
  });
}

export function convertFromConnectionConfig(
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

export function convertFromLocationConstraint(
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

export function convertFromGeographicConstraint(
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

export function convertFromTunnelOptions(tunnelOptions: grpcTypes.TunnelOptions.AsObject): ITunnelOptions {
  return {
    openvpn: {
      mssfix: tunnelOptions.openvpn!.mssfix,
    },
    wireguard: {
      mtu: tunnelOptions.wireguard!.mtu,
      quantumResistant: convertFromQuantumResistantState(
        tunnelOptions.wireguard?.quantumResistant?.state,
      ),
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

export function convertFromQuantumResistantState(
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

export function convertFromObfuscationSettings(
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
  }

  return {
    selectedObfuscation: selectedObfuscationType,
    udp2tcpSettings: obfuscationSettings?.udp2tcp
      ? { port: convertFromConstraint(obfuscationSettings.udp2tcp.port) }
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

  // Handle unknown daemon events
  const keys = Object.entries(data.toObject())
    .filter(([, value]) => value !== undefined)
    .map(([key]) => key);
  throw new Error(`Unknown daemon event received containing ${keys}`);
}

export function convertFromOwnership(ownership: grpcTypes.Ownership): Ownership {
  switch (ownership) {
    case grpcTypes.Ownership.ANY:
      return Ownership.any;
    case grpcTypes.Ownership.MULLVAD_OWNED:
      return Ownership.mullvadOwned;
    case grpcTypes.Ownership.RENTED:
      return Ownership.rented;
  }
}

export function convertToOwnership(ownership: Ownership): grpcTypes.Ownership {
  switch (ownership) {
    case Ownership.any:
      return grpcTypes.Ownership.ANY;
    case Ownership.mullvadOwned:
      return grpcTypes.Ownership.MULLVAD_OWNED;
    case Ownership.rented:
      return grpcTypes.Ownership.RENTED;
  }
}

export function convertFromOpenVpnConstraints(
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

export function convertFromWireguardConstraints(
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

export function convertFromTunnelTypeConstraint(
  constraint: grpcTypes.TunnelType | undefined,
): Constraint<TunnelProtocol> {
  switch (constraint) {
    case grpcTypes.TunnelType.WIREGUARD: {
      return { only: 'wireguard' };
    }
    case grpcTypes.TunnelType.OPENVPN: {
      return { only: 'openvpn' };
    }
    default: {
      return 'any';
    }
  }
}

export function convertFromConstraint<T>(value: T | undefined): Constraint<T> {
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

  if (constraints.tunnelProtocol !== 'any') {
    relayConstraints.setTunnelType(convertToTunnelType(constraints.tunnelProtocol.only));
  }
  relayConstraints.setLocation(convertToLocation(liftConstraint(constraints.location)));
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
  normalBridgeSettings.setLocation(convertToLocation(liftConstraint(constraints.location)));
  normalBridgeSettings.setProvidersList(constraints.providers);

  return normalBridgeSettings;
}

export function convertToLocation(
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

export function convertToGeographicConstraint(
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

export function convertToTunnelType(tunnelProtocol: TunnelProtocol): grpcTypes.TunnelType {
  switch (tunnelProtocol) {
    case 'wireguard':
      return grpcTypes.TunnelType.WIREGUARD;
    case 'openvpn':
      return grpcTypes.TunnelType.OPENVPN;
  }
}

export function convertToOpenVpnConstraints(
  constraints: Partial<IOpenVpnConstraints> | undefined,
): grpcTypes.OpenvpnConstraints | undefined {
  const openvpnConstraints = new grpcTypes.OpenvpnConstraints();
  if (constraints) {
    const protocol = liftConstraint(constraints.protocol);
    if (protocol) {
      const portConstraints = new grpcTypes.TransportPort();
      const port = liftConstraint(constraints.port);
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

export function convertToWireguardConstraints(
  constraint: Partial<IWireguardConstraints> | undefined,
): grpcTypes.WireguardConstraints | undefined {
  if (constraint) {
    const wireguardConstraints = new grpcTypes.WireguardConstraints();

    const port = liftConstraint(constraint.port);
    if (port) {
      wireguardConstraints.setPort(port);
    }

    const ipVersion = liftConstraint(constraint.ipVersion);
    if (ipVersion) {
      const ipVersionProtocol =
        ipVersion === 'ipv4' ? grpcTypes.IpVersion.V4 : grpcTypes.IpVersion.V6;
      wireguardConstraints.setIpVersion(ipVersionProtocol);
    }

    if (constraint.useMultihop) {
      wireguardConstraints.setUseMultihop(constraint.useMultihop);
    }

    const entryLocation = liftConstraint(constraint.entryLocation);
    if (entryLocation) {
      const entryLocationConstraint = convertToLocation(entryLocation);
      wireguardConstraints.setEntryLocation(entryLocationConstraint);
    }

    return wireguardConstraints;
  }
  return undefined;
}

export function convertToTransportProtocol(protocol: RelayProtocol): grpcTypes.TransportProtocol {
  switch (protocol) {
    case 'udp':
      return grpcTypes.TransportProtocol.UDP;
    case 'tcp':
      return grpcTypes.TransportProtocol.TCP;
  }
}

export function convertFromDeviceEvent(deviceEvent: grpcTypes.DeviceEvent): DeviceEvent {
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
          accountToken: accountAndDevice.getAccountToken(),
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

export function convertFromDeviceRemoval(deviceRemoval: grpcTypes.RemoveDeviceEvent): Array<IDevice> {
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

export function convertFromCustomListSettings(
  customListSettings?: grpcTypes.CustomListSettings,
): CustomLists {
  return customListSettings ? convertFromCustomLists(customListSettings.getCustomListsList()) : [];
}

export function convertFromCustomLists(customLists: Array<grpcTypes.CustomList>): CustomLists {
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

export function ensureExists<T>(value: T | undefined, errorMessage: string): T {
  if (value) {
    return value;
  }
  throw new Error(errorMessage);
}
