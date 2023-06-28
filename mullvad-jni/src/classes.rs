pub const CLASSES: &[&str] = &[
    "java/lang/Boolean",
    "java/net/InetAddress",
    "java/net/InetSocketAddress",
    "java/util/ArrayList",
    "net/mullvad/mullvadvpn/model/AccountAndDevice",
    "net/mullvad/mullvadvpn/model/AccountData",
    "net/mullvad/mullvadvpn/model/AppVersionInfo",
    "net/mullvad/mullvadvpn/model/Constraint$Any",
    "net/mullvad/mullvadvpn/model/Constraint$Only",
    "net/mullvad/mullvadvpn/model/DnsState",
    "net/mullvad/mullvadvpn/model/DnsOptions",
    "net/mullvad/mullvadvpn/model/CustomDnsOptions",
    "net/mullvad/mullvadvpn/model/DefaultDnsOptions",
    "net/mullvad/mullvadvpn/model/Device",
    "net/mullvad/mullvadvpn/model/DeviceEvent",
    "net/mullvad/mullvadvpn/model/DeviceEventCause",
    "net/mullvad/mullvadvpn/model/DevicePort",
    "net/mullvad/mullvadvpn/model/DeviceState$LoggedIn",
    "net/mullvad/mullvadvpn/model/DeviceState$LoggedOut",
    "net/mullvad/mullvadvpn/model/DeviceState$Revoked",
    "net/mullvad/mullvadvpn/model/RemoveDeviceEvent",
    "net/mullvad/mullvadvpn/model/GeographicLocationConstraint",
    "net/mullvad/mullvadvpn/model/GeographicLocationConstraint$City",
    "net/mullvad/mullvadvpn/model/GeographicLocationConstraint$Country",
    "net/mullvad/mullvadvpn/model/GeographicLocationConstraint$Hostname",
    "net/mullvad/mullvadvpn/model/GeoIpLocation",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$Ok",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$InvalidAccount",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$RpcError",
    "net/mullvad/mullvadvpn/model/GetAccountDataResult$OtherError",
    "net/mullvad/mullvadvpn/model/LocationConstraint",
    "net/mullvad/mullvadvpn/model/LocationConstraint$Location",
    "net/mullvad/mullvadvpn/model/LocationConstraint$CustomList",
    "net/mullvad/mullvadvpn/model/ObfuscationSettings",
    "net/mullvad/mullvadvpn/model/PublicKey",
    "net/mullvad/mullvadvpn/model/QuantumResistantState",
    "net/mullvad/mullvadvpn/model/Relay",
    "net/mullvad/mullvadvpn/model/RelayConstraints",
    "net/mullvad/mullvadvpn/model/RelayEndpointData$Bridge",
    "net/mullvad/mullvadvpn/model/RelayEndpointData$Openvpn",
    "net/mullvad/mullvadvpn/model/RelayEndpointData$Wireguard",
    "net/mullvad/mullvadvpn/model/RelayList",
    "net/mullvad/mullvadvpn/model/RelayListCity",
    "net/mullvad/mullvadvpn/model/RelayListCountry",
    "net/mullvad/mullvadvpn/model/RelaySettings$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettings$Normal",
    "net/mullvad/mullvadvpn/model/RelaySettingsUpdate$CustomTunnelEndpoint",
    "net/mullvad/mullvadvpn/model/RelaySettingsUpdate$Normal",
    "net/mullvad/mullvadvpn/model/RelayConstraintsUpdate",
    "net/mullvad/mullvadvpn/model/SelectedObfuscation",
    "net/mullvad/mullvadvpn/model/Settings",
    "net/mullvad/mullvadvpn/model/TunnelState$Error",
    "net/mullvad/mullvadvpn/model/TunnelState$Connected",
    "net/mullvad/mullvadvpn/model/TunnelState$Connecting",
    "net/mullvad/mullvadvpn/model/TunnelState$Disconnected",
    "net/mullvad/mullvadvpn/model/TunnelState$Disconnecting",
    "net/mullvad/mullvadvpn/model/Udp2TcpObfuscationSettings",
    "net/mullvad/mullvadvpn/model/VoucherSubmission",
    "net/mullvad/mullvadvpn/model/VoucherSubmissionResult",
    "net/mullvad/mullvadvpn/model/LoginResult",
    "net/mullvad/mullvadvpn/model/WireguardRelayEndpointData",
    "net/mullvad/mullvadvpn/service/MullvadDaemon",
    "net/mullvad/mullvadvpn/service/MullvadVpnService",
    "net/mullvad/talpid/net/Endpoint",
    "net/mullvad/talpid/net/TransportProtocol",
    "net/mullvad/talpid/net/TunnelEndpoint",
    "net/mullvad/talpid/net/ObfuscationEndpoint",
    "net/mullvad/talpid/net/ObfuscationType",
    "net/mullvad/talpid/tun_provider/InetNetwork",
    "net/mullvad/talpid/tun_provider/TunConfig",
    "net/mullvad/talpid/tunnel/ActionAfterDisconnect",
    "net/mullvad/talpid/tunnel/ErrorState",
    "net/mullvad/talpid/tunnel/ErrorStateCause$AuthFailed",
    "net/mullvad/talpid/tunnel/ErrorStateCause$Ipv6Unavailable",
    "net/mullvad/talpid/tunnel/ErrorStateCause$SetFirewallPolicyError",
    "net/mullvad/talpid/tunnel/ErrorStateCause$SetDnsError",
    "net/mullvad/talpid/tunnel/ErrorStateCause$StartTunnelError",
    "net/mullvad/talpid/tunnel/ErrorStateCause$TunnelParameterError",
    "net/mullvad/talpid/tunnel/ErrorStateCause$IsOffline",
    "net/mullvad/talpid/tunnel/ErrorStateCause$InvalidDnsServers",
    "net/mullvad/talpid/tunnel/ErrorStateCause$VpnPermissionDenied",
    "net/mullvad/talpid/tunnel/ParameterGenerationError",
    "net/mullvad/talpid/ConnectivityListener",
    "net/mullvad/talpid/CreateTunResult$Success",
    "net/mullvad/talpid/CreateTunResult$InvalidDnsServers",
    "net/mullvad/talpid/CreateTunResult$PermissionDenied",
    "net/mullvad/talpid/CreateTunResult$TunnelDeviceError",
    "net/mullvad/talpid/TalpidVpnService",
];
