package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.Uri
import io.grpc.ConnectivityState
import java.net.InetAddress
import java.net.InetSocketAddress
import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListsSettings
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.Relay
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelayEndpointType
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelayListCity
import net.mullvad.mullvadvpn.model.RelayListCountry
import net.mullvad.mullvadvpn.model.RelayOverride
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelOptions
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardEndpointData
import net.mullvad.mullvadvpn.model.WireguardRelayEndpointData
import net.mullvad.mullvadvpn.model.WireguardTunnelOptions
import net.mullvad.talpid.net.Endpoint
import net.mullvad.talpid.net.ObfuscationEndpoint
import net.mullvad.talpid.net.ObfuscationType
import net.mullvad.talpid.net.TransportProtocol
import net.mullvad.talpid.net.TunnelEndpoint
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.FirewallPolicyError
import net.mullvad.talpid.tunnel.ParameterGenerationError

internal fun ManagementInterface.TunnelState.toDomain(): TunnelState =
    when (stateCase!!) {
        ManagementInterface.TunnelState.StateCase.DISCONNECTED ->
            TunnelState.Disconnected(
                location = disconnected.disconnectedLocation.toDomain(),
            )
        ManagementInterface.TunnelState.StateCase.CONNECTING ->
            TunnelState.Connecting(
                endpoint = connecting.relayInfo.tunnelEndpoint.toDomain(),
                location = connecting.relayInfo.location.toDomain(),
            )
        ManagementInterface.TunnelState.StateCase.CONNECTED ->
            TunnelState.Connected(
                endpoint = connected.relayInfo.tunnelEndpoint.toDomain(),
                location = connected.relayInfo.location.toDomain(),
            )
        ManagementInterface.TunnelState.StateCase.DISCONNECTING ->
            TunnelState.Disconnecting(
                actionAfterDisconnect = disconnecting.afterDisconnect.toDomain(),
            )
        ManagementInterface.TunnelState.StateCase.ERROR ->
            TunnelState.Error(errorState = error.errorState.toDomain())
        ManagementInterface.TunnelState.StateCase.STATE_NOT_SET ->
            TunnelState.Disconnected(
                location = disconnected.disconnectedLocation.toDomain(),
            )
    }

internal fun ManagementInterface.GeoIpLocation.toDomain(): GeoIpLocation =
    GeoIpLocation(
        ipv4 = InetAddress.getByName(this.ipv4),
        ipv6 = InetAddress.getByName(this.ipv6),
        country = this.country,
        city = this.city,
        latitude = this.latitude,
        longitude = this.longitude,
        hostname = this.hostname
    )

internal fun ManagementInterface.TunnelEndpoint.toDomain(): TunnelEndpoint =
    TunnelEndpoint(
        endpoint =
            with(address.split(":")) {
                Endpoint(
                    address = InetSocketAddress(this[0], this[1].toInt()),
                    protocol = this@toDomain.protocol.toDomain()
                )
            },
        quantumResistant = this.quantumResistant,
        obfuscation = this.obfuscation.toDomain()
    )

internal fun ManagementInterface.ObfuscationEndpoint.toDomain(): ObfuscationEndpoint =
    ObfuscationEndpoint(
        endpoint =
            Endpoint(
                address = InetSocketAddress(address, port),
                protocol = this.protocol.toDomain()
            ),
        obfuscationType = this.obfuscationType.toDomain()
    )

internal fun ManagementInterface.ObfuscationType.toDomain(): ObfuscationType =
    when (this) {
        ManagementInterface.ObfuscationType.UDP2TCP -> ObfuscationType.Udp2Tcp
        ManagementInterface.ObfuscationType.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized obfuscation type")
    }

internal fun ManagementInterface.Endpoint.toDomain(): Endpoint =
    Endpoint(
        address = with(Uri.parse(this.address)) { InetSocketAddress(host, port) },
        protocol = this.protocol.toDomain()
    )

internal fun ManagementInterface.TransportProtocol.toDomain(): TransportProtocol =
    when (this) {
        ManagementInterface.TransportProtocol.TCP -> TransportProtocol.Tcp
        ManagementInterface.TransportProtocol.UDP -> TransportProtocol.Udp
        ManagementInterface.TransportProtocol.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized transport protocol")
    }

internal fun ManagementInterface.AfterDisconnect.toDomain(): ActionAfterDisconnect =
    when (this) {
        ManagementInterface.AfterDisconnect.NOTHING -> ActionAfterDisconnect.Nothing
        ManagementInterface.AfterDisconnect.RECONNECT -> ActionAfterDisconnect.Reconnect
        ManagementInterface.AfterDisconnect.BLOCK -> ActionAfterDisconnect.Block
        ManagementInterface.AfterDisconnect.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized action after disconnect")
    }

internal fun ManagementInterface.ErrorState.toDomain(): ErrorState =
    ErrorState(
        cause =
            when (cause!!) {
                ManagementInterface.ErrorState.Cause.AUTH_FAILED ->
                    ErrorStateCause.AuthFailed(authFailedError.name)
                ManagementInterface.ErrorState.Cause.IPV6_UNAVAILABLE ->
                    ErrorStateCause.Ipv6Unavailable
                ManagementInterface.ErrorState.Cause.SET_FIREWALL_POLICY_ERROR ->
                    ErrorStateCause.SetFirewallPolicyError(policyError.toDomain())
                ManagementInterface.ErrorState.Cause.SET_DNS_ERROR -> ErrorStateCause.SetDnsError
                ManagementInterface.ErrorState.Cause.START_TUNNEL_ERROR ->
                    ErrorStateCause.StartTunnelError
                ManagementInterface.ErrorState.Cause.TUNNEL_PARAMETER_ERROR ->
                    ErrorStateCause.TunnelParameterError(parameterError.toDomain())
                ManagementInterface.ErrorState.Cause.IS_OFFLINE -> ErrorStateCause.IsOffline
                ManagementInterface.ErrorState.Cause.VPN_PERMISSION_DENIED ->
                    ErrorStateCause.VpnPermissionDenied
                ManagementInterface.ErrorState.Cause.SPLIT_TUNNEL_ERROR ->
                    ErrorStateCause.StartTunnelError
                ManagementInterface.ErrorState.Cause.UNRECOGNIZED,
                ManagementInterface.ErrorState.Cause.CREATE_TUNNEL_DEVICE ->
                    throw IllegalArgumentException("Unrecognized error state cause")
            },
        isBlocking = this.hasBlockingError()
    )

internal fun ManagementInterface.ErrorState.FirewallPolicyError.toDomain(): FirewallPolicyError =
    when (this.type!!) {
        ManagementInterface.ErrorState.FirewallPolicyError.ErrorType.GENERIC ->
            FirewallPolicyError.Generic
        ManagementInterface.ErrorState.FirewallPolicyError.ErrorType.LOCKED,
        ManagementInterface.ErrorState.FirewallPolicyError.ErrorType.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized firewall policy error")
    }

internal fun ManagementInterface.ErrorState.GenerationError.toDomain(): ParameterGenerationError =
    when (this) {
        ManagementInterface.ErrorState.GenerationError.NO_MATCHING_RELAY ->
            ParameterGenerationError.NoMatchingRelay
        ManagementInterface.ErrorState.GenerationError.NO_MATCHING_BRIDGE_RELAY ->
            ParameterGenerationError.NoMatchingBridgeRelay
        ManagementInterface.ErrorState.GenerationError.NO_WIREGUARD_KEY ->
            ParameterGenerationError.NoWireguardKey
        ManagementInterface.ErrorState.GenerationError.CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR ->
            ParameterGenerationError.CustomTunnelHostResultionError
        ManagementInterface.ErrorState.GenerationError.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized parameter generation error")
    }

internal fun ManagementInterface.Settings.toDomain(): Settings =
    Settings(
        relaySettings = relaySettings.toDomain(),
        obfuscationSettings = obfuscationSettings.toDomain(),
        customLists = CustomListsSettings(customLists.customListsList.map { it.toDomain() }),
        allowLan = allowLan,
        autoConnect = autoConnect,
        tunnelOptions = tunnelOptions.toDomain(),
        relayOverrides = relayOverridesList.map { it.toDomain() },
        showBetaReleases = showBetaReleases
    )

internal fun ManagementInterface.RelayOverride.toDomain(): RelayOverride =
    RelayOverride(
        hostname = hostname,
        ipv4AddressIn = if (hasIpv4AddrIn()) InetAddress.getByName(ipv4AddrIn) else null,
        ipv6AddressIn = if (hasIpv6AddrIn()) InetAddress.getByName(ipv6AddrIn) else null
    )

internal fun ManagementInterface.RelaySettings.toDomain(): RelaySettings =
    when (endpointCase) {
        ManagementInterface.RelaySettings.EndpointCase.CUSTOM -> RelaySettings.CustomTunnelEndpoint
        ManagementInterface.RelaySettings.EndpointCase.NORMAL ->
            RelaySettings.Normal(this.normal.toDomain())
        ManagementInterface.RelaySettings.EndpointCase.ENDPOINT_NOT_SET ->
            throw IllegalArgumentException("RelaySettings endpoint not set")
    }

internal fun ManagementInterface.NormalRelaySettings.toDomain(): RelayConstraints =
    RelayConstraints(
        location = location.toDomain(),
        providers = providersList.toDomain(),
        ownership = ownership.toDomain(),
        wireguardConstraints = wireguardConstraints.toDomain()
    )

internal fun ManagementInterface.LocationConstraint.toDomain(): Constraint<LocationConstraint> =
    when (typeCase) {
        ManagementInterface.LocationConstraint.TypeCase.CUSTOM_LIST ->
            Constraint.Only(LocationConstraint.CustomList(CustomListId(customList)))
        ManagementInterface.LocationConstraint.TypeCase.LOCATION ->
            Constraint.Only(LocationConstraint.Location(location.toDomain()))
        ManagementInterface.LocationConstraint.TypeCase.TYPE_NOT_SET -> Constraint.Any()
    }

internal fun Constraint<LocationConstraint>.fromDomain(): ManagementInterface.LocationConstraint =
    when (this) {
        is Constraint.Any ->
            ManagementInterface.LocationConstraint.newBuilder()
                .setLocation(ManagementInterface.GeographicLocationConstraint.getDefaultInstance())
                .build()
        is Constraint.Only ->
            when (this.value) {
                is LocationConstraint.CustomList ->
                    ManagementInterface.LocationConstraint.newBuilder()
                        .setCustomList((this.value as LocationConstraint.CustomList).listId.value)
                        .build()
                is LocationConstraint.Location ->
                    ManagementInterface.LocationConstraint.newBuilder()
                        .setLocation(
                            (this.value as LocationConstraint.Location).location.fromDomain()
                        )
                        .build()
            }
    }

internal fun ManagementInterface.GeographicLocationConstraint.toDomain():
    GeographicLocationConstraint =
    when {
        hasHostname() && hasCity() -> GeographicLocationConstraint.Hostname(country, city, hostname)
        hasCity() -> GeographicLocationConstraint.City(country, city)
        else -> GeographicLocationConstraint.Country(country)
    }

internal fun List<String>.toDomain(): Constraint<Providers> =
    if (isEmpty()) Constraint.Any() else Constraint.Only(Providers(HashSet(this)))

internal fun ManagementInterface.WireguardConstraints.toDomain(): WireguardConstraints =
    WireguardConstraints(
        port =
            if (hasPort()) {
                Constraint.Any()
            } else {
                Constraint.Only(Port(port))
            },
    )

internal fun ManagementInterface.Ownership.toDomain(): Constraint<Ownership> =
    when (this) {
        ManagementInterface.Ownership.ANY -> Constraint.Any()
        ManagementInterface.Ownership.MULLVAD_OWNED -> Constraint.Only(Ownership.MullvadOwned)
        ManagementInterface.Ownership.RENTED -> Constraint.Only(Ownership.Rented)
        ManagementInterface.Ownership.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized ownership")
    }

internal fun ManagementInterface.ObfuscationSettings.toDomain(): ObfuscationSettings =
    ObfuscationSettings(
        selectedObfuscation = selectedObfuscation.toDomain(),
        udp2tcp = this.udp2Tcp.toDomain()
    )

internal fun ManagementInterface.ObfuscationSettings.SelectedObfuscation.toDomain():
    SelectedObfuscation =
    when (this) {
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.AUTO -> SelectedObfuscation.Auto
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.OFF -> SelectedObfuscation.Off
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.UDP2TCP ->
            SelectedObfuscation.Udp2Tcp
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized selected obfuscation")
    }

internal fun ManagementInterface.Udp2TcpObfuscationSettings.toDomain(): Udp2TcpObfuscationSettings =
    if (this.hasPort()) {
        Udp2TcpObfuscationSettings(Constraint.Only(Port(port)))
    } else {
        Udp2TcpObfuscationSettings(Constraint.Any())
    }

internal fun ManagementInterface.CustomList.toDomain(): CustomList =
    CustomList(id = CustomListId(id), name = name, locations = locationsList.map { it.toDomain() })

internal fun ManagementInterface.TunnelOptions.toDomain(): TunnelOptions =
    TunnelOptions(wireguard = wireguard.toDomain(), dnsOptions = dnsOptions.toDomain())

internal fun ManagementInterface.TunnelOptions.WireguardOptions.toDomain(): WireguardTunnelOptions =
    WireguardTunnelOptions(
        mtu = if (hasMtu()) mtu else null,
        quantumResistant = this.quantumResistant.toDomain(),
    )

internal fun ManagementInterface.QuantumResistantState.toDomain(): QuantumResistantState =
    when (state) {
        ManagementInterface.QuantumResistantState.State.AUTO -> QuantumResistantState.Auto
        ManagementInterface.QuantumResistantState.State.ON -> QuantumResistantState.On
        ManagementInterface.QuantumResistantState.State.OFF -> QuantumResistantState.Off
        ManagementInterface.QuantumResistantState.State.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized quantum resistant state")
    }

internal fun ManagementInterface.DnsOptions.toDomain(): DnsOptions =
    DnsOptions(
        state = this.state.toDomain(),
        defaultOptions = defaultOptions.toDomain(),
        customOptions = customOptions.toDomain()
    )

internal fun ManagementInterface.DnsOptions.DnsState.toDomain(): DnsState =
    when (this) {
        ManagementInterface.DnsOptions.DnsState.DEFAULT -> DnsState.Default
        ManagementInterface.DnsOptions.DnsState.CUSTOM -> DnsState.Custom
        ManagementInterface.DnsOptions.DnsState.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized dns state")
    }

internal fun ManagementInterface.DefaultDnsOptions.toDomain() =
    DefaultDnsOptions(
        blockAds = blockAds,
        blockMalware = blockMalware,
        blockAdultContent = blockAdultContent,
        blockGambling = blockGambling,
        blockSocialMedia = blockSocialMedia,
        blockTrackers = blockTrackers
    )

internal fun ManagementInterface.CustomDnsOptions.toDomain() =
    CustomDnsOptions(this.addressesList.map { InetAddress.getByName(it) })

internal fun DnsOptions.toDomain(): ManagementInterface.DnsOptions =
    ManagementInterface.DnsOptions.newBuilder()
        .setState(this.state.toDomain())
        .setCustomOptions(this.customOptions.fromDomain())
        .setDefaultOptions(this.defaultOptions.fromDomain())
        .build()

internal fun DnsState.toDomain(): ManagementInterface.DnsOptions.DnsState =
    when (this) {
        DnsState.Default -> ManagementInterface.DnsOptions.DnsState.DEFAULT
        DnsState.Custom -> ManagementInterface.DnsOptions.DnsState.CUSTOM
    }

internal fun CustomDnsOptions.fromDomain(): ManagementInterface.CustomDnsOptions =
    ManagementInterface.CustomDnsOptions.newBuilder()
        .addAllAddresses(this.addresses.map { it.toString() })
        .build()

internal fun DefaultDnsOptions.fromDomain(): ManagementInterface.DefaultDnsOptions =
    ManagementInterface.DefaultDnsOptions.newBuilder()
        .setBlockAds(blockAds)
        .setBlockGambling(blockGambling)
        .setBlockMalware(blockMalware)
        .setBlockTrackers(blockTrackers)
        .setBlockAdultContent(blockAdultContent)
        .setBlockSocialMedia(blockSocialMedia)
        .build()

internal fun QuantumResistantState.toDomain(): ManagementInterface.QuantumResistantState =
    ManagementInterface.QuantumResistantState.newBuilder()
        .setState(
            when (this) {
                QuantumResistantState.Auto -> ManagementInterface.QuantumResistantState.State.AUTO
                QuantumResistantState.On -> ManagementInterface.QuantumResistantState.State.ON
                QuantumResistantState.Off -> ManagementInterface.QuantumResistantState.State.OFF
            }
        )
        .build()

internal fun ObfuscationSettings.fromDomain(): ManagementInterface.ObfuscationSettings =
    ManagementInterface.ObfuscationSettings.newBuilder()
        .setSelectedObfuscation(this.selectedObfuscation.toDomain())
        .setUdp2Tcp(this.udp2tcp.toDomain())
        .build()

internal fun SelectedObfuscation.toDomain():
    ManagementInterface.ObfuscationSettings.SelectedObfuscation =
    when (this) {
        SelectedObfuscation.Udp2Tcp ->
            ManagementInterface.ObfuscationSettings.SelectedObfuscation.UDP2TCP
        SelectedObfuscation.Auto -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.AUTO
        SelectedObfuscation.Off -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.OFF
    }

internal fun Udp2TcpObfuscationSettings.toDomain(): ManagementInterface.Udp2TcpObfuscationSettings =
    when (val port = this.port) {
        is Constraint.Any ->
            ManagementInterface.Udp2TcpObfuscationSettings.newBuilder().clearPort().build()
        is Constraint.Only ->
            ManagementInterface.Udp2TcpObfuscationSettings.newBuilder()
                .setPort(port.value.value)
                .build()
    }

internal fun ManagementInterface.AppVersionInfo.toDomain(): AppVersionInfo =
    AppVersionInfo(supported = this.supported, suggestedUpgrade = this.suggestedUpgrade)

internal fun ConnectivityState.toDomain(): GrpcConnectivityState =
    when (this) {
        ConnectivityState.CONNECTING -> GrpcConnectivityState.Connecting
        ConnectivityState.READY -> GrpcConnectivityState.Ready
        ConnectivityState.IDLE -> GrpcConnectivityState.Idle
        ConnectivityState.TRANSIENT_FAILURE -> GrpcConnectivityState.TransientFailure
        ConnectivityState.SHUTDOWN -> GrpcConnectivityState.Shutdown
    }

fun LocationConstraint.fromDomain(): ManagementInterface.LocationConstraint =
    when (this) {
        is LocationConstraint.CustomList ->
            ManagementInterface.LocationConstraint.newBuilder()
                .setCustomList(this.listId.value)
                .build()
        is LocationConstraint.Location ->
            ManagementInterface.LocationConstraint.newBuilder()
                .setLocation(this.location.fromDomain())
                .build()
    }

internal fun GeographicLocationConstraint.fromDomain():
    ManagementInterface.GeographicLocationConstraint =
    when (this) {
        is GeographicLocationConstraint.Country ->
            ManagementInterface.GeographicLocationConstraint.newBuilder()
                .setCountry(this.countryCode)
                .build()
        is GeographicLocationConstraint.City ->
            ManagementInterface.GeographicLocationConstraint.newBuilder()
                .setCountry(this.countryCode)
                .setCity(this.cityCode)
                .build()
        is GeographicLocationConstraint.Hostname ->
            ManagementInterface.GeographicLocationConstraint.newBuilder()
                .setCountry(this.countryCode)
                .setCity(this.cityCode)
                .setHostname(this.hostname)
                .build()
    }

internal fun CustomList.fromDomain(): ManagementInterface.CustomList =
    ManagementInterface.CustomList.newBuilder()
        .setId(this.id.value)
        .setName(this.name)
        .addAllLocations(this.locations.map { it.fromDomain() })
        .build()

internal fun ManagementInterface.RelayList.toDomain(): RelayList =
    RelayList(
        countries = countriesList.map { it.toDomain() },
        wireguardEndpointData =
            WireguardEndpointData(
                portRanges = this.wireguard.portRangesList.map { it.toDomain() },
            )
    )

internal fun ManagementInterface.RelayListCountry.toDomain(): RelayListCountry =
    RelayListCountry(
        name = name,
        code = code,
        cities = citiesList.map { it.toDomain() },
    )

internal fun ManagementInterface.RelayListCity.toDomain(): RelayListCity =
    RelayListCity(name = name, code = code, relays = relaysList.map { it.toDomain() })

internal fun ManagementInterface.Relay.toDomain(): Relay =
    Relay(
        hostname = hostname,
        active = active,
        owned = owned,
        provider = provider,
        endpointType =
            when (endpointType) {
                ManagementInterface.Relay.RelayType.OPENVPN -> RelayEndpointType.Openvpn
                ManagementInterface.Relay.RelayType.BRIDGE -> RelayEndpointType.Bridge
                ManagementInterface.Relay.RelayType.WIREGUARD ->
                    RelayEndpointType.Wireguard(
                        wireguardRelayEndpointData = WireguardRelayEndpointData
                    )
                ManagementInterface.Relay.RelayType.UNRECOGNIZED ->
                    throw IllegalArgumentException("Unrecognized relay type")
            }
    )

internal fun ManagementInterface.PortRange.toDomain(): PortRange = PortRange(first, last)
