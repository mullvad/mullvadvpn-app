@file:Suppress("TooManyFunctions")

package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import io.grpc.ConnectivityState
import java.net.InetAddress
import java.net.InetSocketAddress
import java.time.Instant
import java.time.ZoneId
import java.util.UUID
import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.lib.daemon.grpc.GrpcConnectivityState
import net.mullvad.mullvadvpn.lib.daemon.grpc.RelayNameComparator
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountId
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.model.AppVersionInfo
import net.mullvad.mullvadvpn.lib.model.AuthFailedError
import net.mullvad.mullvadvpn.lib.model.Cipher
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.DaitaSettings
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.Endpoint
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationEndpoint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.ObfuscationType
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherSuccess
import net.mullvad.mullvadvpn.lib.model.RelayConstraints
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayList
import net.mullvad.mullvadvpn.lib.model.RelayOverride
import net.mullvad.mullvadvpn.lib.model.RelaySettings
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksSettings
import net.mullvad.mullvadvpn.lib.model.SocksAuth
import net.mullvad.mullvadvpn.lib.model.SplitTunnelSettings
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint
import net.mullvad.mullvadvpn.lib.model.TunnelOptions
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData
import net.mullvad.mullvadvpn.lib.model.WireguardRelayEndpointData
import net.mullvad.mullvadvpn.lib.model.WireguardTunnelOptions

internal fun ManagementInterface.TunnelState.toDomain(): TunnelState =
    when (stateCase!!) {
        ManagementInterface.TunnelState.StateCase.DISCONNECTED -> disconnected.toDomain()
        ManagementInterface.TunnelState.StateCase.CONNECTING -> connecting.toDomain()
        ManagementInterface.TunnelState.StateCase.CONNECTED -> connected.toDomain()
        ManagementInterface.TunnelState.StateCase.DISCONNECTING -> disconnecting.toDomain()
        ManagementInterface.TunnelState.StateCase.ERROR -> error.toDomain()
        ManagementInterface.TunnelState.StateCase.STATE_NOT_SET ->
            TunnelState.Disconnected(location = disconnected.disconnectedLocation.toDomain())
    }

private fun ManagementInterface.TunnelState.Connecting.toDomain(): TunnelState.Connecting =
    TunnelState.Connecting(
        endpoint = relayInfo.tunnelEndpoint.toDomain(),
        location =
            if (relayInfo.hasLocation()) {
                relayInfo.location.toDomain()
            } else null,
        featureIndicators = featureIndicators.toDomain(),
    )

private fun ManagementInterface.TunnelState.Disconnected.toDomain(): TunnelState.Disconnected =
    TunnelState.Disconnected(
        location =
            if (hasDisconnectedLocation()) {
                disconnectedLocation.toDomain()
            } else null
    )

private fun ManagementInterface.TunnelState.Connected.toDomain(): TunnelState.Connected =
    TunnelState.Connected(
        endpoint = relayInfo.tunnelEndpoint.toDomain(),
        location =
            if (relayInfo.hasLocation()) {
                relayInfo.location.toDomain()
            } else {
                null
            },
        featureIndicators = featureIndicators.toDomain(),
    )

private fun ManagementInterface.TunnelState.Disconnecting.toDomain(): TunnelState.Disconnecting =
    TunnelState.Disconnecting(actionAfterDisconnect = afterDisconnect.toDomain())

private fun ManagementInterface.TunnelState.Error.toDomain(): TunnelState.Error {
    val otherAlwaysOnAppError =
        errorState.let {
            if (it.hasOtherAlwaysOnAppError()) {
                ErrorStateCause.OtherAlwaysOnApp(it.otherAlwaysOnAppError.appName)
            } else {
                null
            }
        }

    val invalidDnsServers =
        errorState.let {
            if (it.hasInvalidDnsServersError()) {
                ErrorStateCause.InvalidDnsServers(
                    it.invalidDnsServersError.ipAddrsList.toList().map { InetAddress.getByName(it) }
                )
            } else {
                null
            }
        }

    return TunnelState.Error(
        errorState =
            errorState.toDomain(
                otherAlwaysOnApp = otherAlwaysOnAppError,
                invalidDnsServers = invalidDnsServers,
            )
    )
}

internal fun ManagementInterface.GeoIpLocation.toDomain(): GeoIpLocation =
    GeoIpLocation(
        ipv4 =
            if (hasIpv4()) {
                InetAddress.getByName(ipv4)
            } else {
                null
            },
        ipv6 =
            if (hasIpv6()) {
                InetAddress.getByName(ipv6)
            } else {
                null
            },
        country = country,
        city = if (hasCity()) city else null,
        latitude = latitude,
        longitude = longitude,
        hostname = if (hasHostname()) hostname else null,
        entryHostname = if (hasEntryHostname()) entryHostname else null,
    )

internal fun ManagementInterface.TunnelEndpoint.toDomain(): TunnelEndpoint =
    TunnelEndpoint(
        endpoint =
            with(address) {
                val indexOfSeparator = indexOfLast { it == ':' }
                val ipPart =
                    address.substring(0, indexOfSeparator).filter { it !in listOf('[', ']') }
                val portPart = address.substring(indexOfSeparator + 1)

                Endpoint(
                    address = InetSocketAddress(InetAddress.getByName(ipPart), portPart.toInt()),
                    protocol = protocol.toDomain(),
                )
            },
        quantumResistant = quantumResistant,
        obfuscation =
            if (hasObfuscation()) {
                obfuscation.toDomain()
            } else {
                null
            },
        daita = daita,
    )

internal fun ManagementInterface.ObfuscationEndpoint.toDomain(): ObfuscationEndpoint =
    ObfuscationEndpoint(
        endpoint =
            Endpoint(address = InetSocketAddress(address, port), protocol = protocol.toDomain()),
        obfuscationType = obfuscationType.toDomain(),
    )

internal fun ManagementInterface.ObfuscationEndpoint.ObfuscationType.toDomain(): ObfuscationType =
    when (this) {
        ManagementInterface.ObfuscationEndpoint.ObfuscationType.UDP2TCP -> ObfuscationType.Udp2Tcp
        ManagementInterface.ObfuscationEndpoint.ObfuscationType.SHADOWSOCKS ->
            ObfuscationType.Shadowsocks
        ManagementInterface.ObfuscationEndpoint.ObfuscationType.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized obfuscation type")
    }

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

internal fun ManagementInterface.ErrorState.toDomain(
    otherAlwaysOnApp: ErrorStateCause.OtherAlwaysOnApp?,
    invalidDnsServers: ErrorStateCause.InvalidDnsServers?,
): ErrorState =
    ErrorState(
        cause =
            when (cause!!) {
                ManagementInterface.ErrorState.Cause.AUTH_FAILED ->
                    ErrorStateCause.AuthFailed(authFailedError.toDomain())
                ManagementInterface.ErrorState.Cause.IPV6_UNAVAILABLE ->
                    ErrorStateCause.Ipv6Unavailable
                ManagementInterface.ErrorState.Cause.SET_FIREWALL_POLICY_ERROR ->
                    policyError.toDomain()
                ManagementInterface.ErrorState.Cause.SET_DNS_ERROR -> ErrorStateCause.DnsError
                ManagementInterface.ErrorState.Cause.START_TUNNEL_ERROR ->
                    ErrorStateCause.StartTunnelError
                ManagementInterface.ErrorState.Cause.TUNNEL_PARAMETER_ERROR ->
                    ErrorStateCause.TunnelParameterError(parameterError.toDomain())
                ManagementInterface.ErrorState.Cause.IS_OFFLINE -> ErrorStateCause.IsOffline
                ManagementInterface.ErrorState.Cause.SPLIT_TUNNEL_ERROR ->
                    ErrorStateCause.StartTunnelError
                ManagementInterface.ErrorState.Cause.UNRECOGNIZED,
                ManagementInterface.ErrorState.Cause.NEED_FULL_DISK_PERMISSIONS,
                ManagementInterface.ErrorState.Cause.CREATE_TUNNEL_DEVICE ->
                    throw IllegalArgumentException("Unrecognized error state cause")
                ManagementInterface.ErrorState.Cause.NOT_PREPARED -> ErrorStateCause.NotPrepared
                ManagementInterface.ErrorState.Cause.OTHER_ALWAYS_ON_APP -> otherAlwaysOnApp!!
                ManagementInterface.ErrorState.Cause.OTHER_LEGACY_ALWAYS_ON_VPN ->
                    ErrorStateCause.OtherLegacyAlwaysOnApp
                ManagementInterface.ErrorState.Cause.INVALID_DNS_SERVERS -> invalidDnsServers!!
            },
        isBlocking = !hasBlockingError(),
    )

private fun ManagementInterface.ErrorState.AuthFailedError.toDomain(): AuthFailedError =
    when (this) {
        ManagementInterface.ErrorState.AuthFailedError.UNKNOWN -> AuthFailedError.Unknown
        ManagementInterface.ErrorState.AuthFailedError.INVALID_ACCOUNT ->
            AuthFailedError.InvalidAccount
        ManagementInterface.ErrorState.AuthFailedError.EXPIRED_ACCOUNT ->
            AuthFailedError.ExpiredAccount
        ManagementInterface.ErrorState.AuthFailedError.TOO_MANY_CONNECTIONS ->
            AuthFailedError.TooManyConnections
        ManagementInterface.ErrorState.AuthFailedError.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized auth failed error")
    }

internal fun ManagementInterface.ErrorState.FirewallPolicyError.toDomain():
    ErrorStateCause.FirewallPolicyError =
    when (type!!) {
        ManagementInterface.ErrorState.FirewallPolicyError.ErrorType.GENERIC ->
            ErrorStateCause.FirewallPolicyError.Generic
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
        customLists = customLists.customListsList.map { it.toDomain() },
        allowLan = allowLan,
        tunnelOptions = tunnelOptions.toDomain(),
        relayOverrides = relayOverridesList.map { it.toDomain() },
        showBetaReleases = showBetaReleases,
        splitTunnelSettings = splitTunnel.toDomain(),
        apiAccessMethodSettings = apiAccessMethods.toDomain(),
    )

internal fun ManagementInterface.RelayOverride.toDomain(): RelayOverride =
    RelayOverride(
        hostname = hostname,
        ipv4AddressIn = if (hasIpv4AddrIn()) InetAddress.getByName(ipv4AddrIn) else null,
        ipv6AddressIn = if (hasIpv6AddrIn()) InetAddress.getByName(ipv6AddrIn) else null,
    )

internal fun ManagementInterface.RelaySettings.toDomain(): RelaySettings =
    when (endpointCase) {
        ManagementInterface.RelaySettings.EndpointCase.CUSTOM ->
            throw IllegalArgumentException("CustomTunnelEndpoint is not supported")
        ManagementInterface.RelaySettings.EndpointCase.NORMAL -> RelaySettings(normal.toDomain())
        ManagementInterface.RelaySettings.EndpointCase.ENDPOINT_NOT_SET ->
            throw IllegalArgumentException("RelaySettings endpoint not set")
        else -> throw NullPointerException("RelaySettings endpoint is null")
    }

internal fun ManagementInterface.NormalRelaySettings.toDomain(): RelayConstraints =
    RelayConstraints(
        location = location.toDomain(),
        providers = providersList.toDomain(),
        ownership = ownership.toDomain(),
        wireguardConstraints = wireguardConstraints.toDomain(),
    )

internal fun ManagementInterface.LocationConstraint.toDomain(): Constraint<RelayItemId> =
    when (typeCase) {
        ManagementInterface.LocationConstraint.TypeCase.CUSTOM_LIST ->
            Constraint.Only(CustomListId(customList))
        ManagementInterface.LocationConstraint.TypeCase.LOCATION ->
            Constraint.Only(location.toDomain())
        ManagementInterface.LocationConstraint.TypeCase.TYPE_NOT_SET -> Constraint.Any
        else -> throw IllegalArgumentException("Location constraint type is null")
    }

@Suppress("ReturnCount")
internal fun ManagementInterface.GeographicLocationConstraint.toDomain(): GeoLocationId {
    val country = GeoLocationId.Country(country)
    if (!hasCity()) {
        return country
    }

    val city = GeoLocationId.City(country, city)
    if (!hasHostname()) {
        return city
    }
    return GeoLocationId.Hostname(city, hostname)
}

internal fun List<String>.toDomain(): Constraint<Providers> =
    if (isEmpty()) Constraint.Any else Constraint.Only(map { ProviderId(it) }.toSet())

internal fun ManagementInterface.WireguardConstraints.toDomain(): WireguardConstraints =
    WireguardConstraints(
        port =
            if (hasPort()) {
                Constraint.Only(Port(port))
            } else {
                Constraint.Any
            },
        isMultihopEnabled = useMultihop,
        entryLocation = entryLocation.toDomain(),
        ipVersion =
            if (hasIpVersion()) {
                Constraint.Only(ipVersion.toDomain())
            } else {
                Constraint.Any
            },
    )

internal fun ManagementInterface.Ownership.toDomain(): Constraint<Ownership> =
    when (this) {
        ManagementInterface.Ownership.ANY -> Constraint.Any
        ManagementInterface.Ownership.MULLVAD_OWNED -> Constraint.Only(Ownership.MullvadOwned)
        ManagementInterface.Ownership.RENTED -> Constraint.Only(Ownership.Rented)
        ManagementInterface.Ownership.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized ownership")
    }

internal fun ManagementInterface.ObfuscationSettings.toDomain(): ObfuscationSettings =
    ObfuscationSettings(
        selectedObfuscationMode = selectedObfuscation.toDomain(),
        udp2tcp = udp2Tcp.toDomain(),
        shadowsocks = shadowsocks.toDomain(),
    )

internal fun ManagementInterface.ObfuscationSettings.SelectedObfuscation.toDomain():
    ObfuscationMode =
    when (this) {
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.AUTO -> ObfuscationMode.Auto
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.OFF -> ObfuscationMode.Off
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.UDP2TCP ->
            ObfuscationMode.Udp2Tcp
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS ->
            ObfuscationMode.Shadowsocks
        ManagementInterface.ObfuscationSettings.SelectedObfuscation.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized selected obfuscation")
    }

internal fun ManagementInterface.Udp2TcpObfuscationSettings.toDomain(): Udp2TcpObfuscationSettings =
    if (hasPort()) {
        Udp2TcpObfuscationSettings(Constraint.Only(Port(port)))
    } else {
        Udp2TcpObfuscationSettings(Constraint.Any)
    }

internal fun ManagementInterface.ShadowsocksSettings.toDomain(): ShadowsocksSettings =
    if (hasPort()) {
        ShadowsocksSettings(Constraint.Only(Port(port)))
    } else {
        ShadowsocksSettings(Constraint.Any)
    }

internal fun ManagementInterface.CustomList.toDomain(): CustomList =
    CustomList(
        id = CustomListId(id),
        name = CustomListName.fromString(name),
        locations = locationsList.map { it.toDomain() },
    )

internal fun ManagementInterface.TunnelOptions.toDomain(): TunnelOptions =
    TunnelOptions(wireguard = wireguard.toDomain(), dnsOptions = dnsOptions.toDomain())

internal fun ManagementInterface.TunnelOptions.WireguardOptions.toDomain(): WireguardTunnelOptions =
    WireguardTunnelOptions(
        mtu = if (hasMtu()) Mtu(mtu) else null,
        quantumResistant = quantumResistant.toDomain(),
        daitaSettings = daita.toDomain(),
    )

internal fun ManagementInterface.DaitaSettings.toDomain(): DaitaSettings =
    DaitaSettings(enabled = enabled, directOnly = directOnly)

internal fun ManagementInterface.QuantumResistantState.toDomain(): QuantumResistantState =
    when (state) {
        ManagementInterface.QuantumResistantState.State.AUTO -> QuantumResistantState.Auto
        ManagementInterface.QuantumResistantState.State.ON -> QuantumResistantState.On
        ManagementInterface.QuantumResistantState.State.OFF -> QuantumResistantState.Off
        ManagementInterface.QuantumResistantState.State.UNRECOGNIZED ->
            throw IllegalArgumentException("Unrecognized quantum resistant state")
        else -> throw NullPointerException("Quantum resistant state is null")
    }

internal fun ManagementInterface.DnsOptions.toDomain(): DnsOptions =
    DnsOptions(
        state = state.toDomain(),
        defaultOptions = defaultOptions.toDomain(),
        customOptions = customOptions.toDomain(),
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
        blockTrackers = blockTrackers,
    )

internal fun ManagementInterface.CustomDnsOptions.toDomain() =
    CustomDnsOptions(addressesList.map { InetAddress.getByName(it) })

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

internal fun ManagementInterface.AppVersionInfo.toDomain(): AppVersionInfo =
    AppVersionInfo(
        supported = supported,
        suggestedUpgrade = if (hasSuggestedUpgrade()) suggestedUpgrade else null,
    )

internal fun ConnectivityState.toDomain(): GrpcConnectivityState =
    when (this) {
        ConnectivityState.CONNECTING -> GrpcConnectivityState.Connecting
        ConnectivityState.READY -> GrpcConnectivityState.Ready
        ConnectivityState.IDLE -> GrpcConnectivityState.Idle
        ConnectivityState.TRANSIENT_FAILURE -> GrpcConnectivityState.TransientFailure
        ConnectivityState.SHUTDOWN -> GrpcConnectivityState.Shutdown
    }

internal fun ManagementInterface.RelayList.toDomain(): RelayList =
    RelayList(countriesList.toDomain(), wireguard.toDomain())

internal fun ManagementInterface.WireguardEndpointData.toDomain(): WireguardEndpointData =
    WireguardEndpointData(
        portRangesList.map { it.toDomain() },
        shadowsocksPortRangesList.map { it.toDomain() },
    )

internal fun ManagementInterface.WireguardRelayEndpointData.toDomain(): WireguardRelayEndpointData =
    WireguardRelayEndpointData(daita)

internal fun ManagementInterface.PortRange.toDomain(): PortRange = PortRange(first..last)

/**
 * Convert from a list of ManagementInterface.RelayListCountry to a model.RelayList. Non-wireguard
 * relays are filtered out. So are also cities that only contains non-wireguard relays and countries
 * that does not have any cities. Countries, cities and relays are ordered by name.
 */
internal fun List<ManagementInterface.RelayListCountry>.toDomain():
    List<RelayItem.Location.Country> =
    map(ManagementInterface.RelayListCountry::toDomain)
        .filter { it.cities.isNotEmpty() }
        .sortedBy { it.name }

internal fun ManagementInterface.RelayListCountry.toDomain(): RelayItem.Location.Country {
    val countryCode = GeoLocationId.Country(code)
    return RelayItem.Location.Country(
        countryCode,
        name,
        citiesList
            .map { city -> city.toDomain(countryCode) }
            .filter { it.relays.isNotEmpty() }
            .sortedBy { it.name },
    )
}

internal fun ManagementInterface.RelayListCity.toDomain(
    countryCode: GeoLocationId.Country
): RelayItem.Location.City {
    val cityCode = GeoLocationId.City(countryCode, code)
    return RelayItem.Location.City(
        name = name,
        id = cityCode,
        relays =
            relaysList
                .filter { it.endpointType == ManagementInterface.Relay.RelayType.WIREGUARD }
                .map { it.toDomain(cityCode) }
                .sortedWith(RelayNameComparator),
    )
}

internal fun ManagementInterface.Relay.toDomain(
    cityCode: GeoLocationId.City
): RelayItem.Location.Relay =
    RelayItem.Location.Relay(
        id = GeoLocationId.Hostname(cityCode, hostname),
        active = active,
        provider = ProviderId(provider),
        ownership = if (owned) Ownership.MullvadOwned else Ownership.Rented,
        daita =
            if (
                hasEndpointData() && endpointType == ManagementInterface.Relay.RelayType.WIREGUARD
            ) {
                ManagementInterface.WireguardRelayEndpointData.parseFrom(endpointData.value).daita
            } else false,
    )

private fun Instant.atDefaultZone() = atZone(ZoneId.systemDefault())

internal fun ManagementInterface.Device.toDomain(): Device =
    Device(DeviceId.fromString(id), name, Instant.ofEpochSecond(created.seconds).atDefaultZone())

internal fun ManagementInterface.DeviceState.toDomain(): DeviceState =
    when (state) {
        ManagementInterface.DeviceState.State.LOGGED_IN ->
            DeviceState.LoggedIn(AccountNumber(device.accountNumber), device.device.toDomain())
        ManagementInterface.DeviceState.State.LOGGED_OUT -> DeviceState.LoggedOut
        ManagementInterface.DeviceState.State.REVOKED -> DeviceState.Revoked
        ManagementInterface.DeviceState.State.UNRECOGNIZED ->
            throw IllegalArgumentException("Non valid device state")
        else -> throw NullPointerException("Device state is null")
    }

internal fun ManagementInterface.AccountData.toDomain(): AccountData =
    AccountData(
        AccountId(UUID.fromString(id)),
        expiryDate = Instant.ofEpochSecond(expiry.seconds).atDefaultZone(),
    )

internal fun ManagementInterface.VoucherSubmission.toDomain(): RedeemVoucherSuccess =
    RedeemVoucherSuccess(
        timeAdded = secondsAdded,
        newExpiryDate = Instant.ofEpochSecond(newExpiry.seconds).atDefaultZone(),
    )

internal fun ManagementInterface.SplitTunnelSettings.toDomain(): SplitTunnelSettings =
    SplitTunnelSettings(
        enabled = enableExclusions,
        excludedApps = appsList.map { AppId(it) }.toSet(),
    )

internal fun ManagementInterface.PlayPurchasePaymentToken.toDomain(): PlayPurchasePaymentToken =
    PlayPurchasePaymentToken(value = token)

internal fun ManagementInterface.ApiAccessMethodSettings.toDomain(): List<ApiAccessMethodSetting> =
    buildList {
        add(direct.toDomain())
        add(mullvadBridges.toDomain())
        add(encryptedDnsProxy.toDomain())
        addAll(customList.map { it.toDomain() })
    }

internal fun ManagementInterface.AccessMethodSetting.toDomain(): ApiAccessMethodSetting =
    ApiAccessMethodSetting(
        id = ApiAccessMethodId.fromString(id.value),
        name = ApiAccessMethodName.fromString(name),
        enabled = enabled,
        apiAccessMethod = accessMethod.toDomain(),
    )

internal fun ManagementInterface.AccessMethod.toDomain(): ApiAccessMethod =
    when {
        hasDirect() -> ApiAccessMethod.Direct
        hasBridges() -> ApiAccessMethod.Bridges
        hasEncryptedDnsProxy() -> ApiAccessMethod.EncryptedDns
        hasCustom() -> custom.toDomain()
        else -> error("Type not found")
    }

internal fun ManagementInterface.CustomProxy.toDomain(): ApiAccessMethod.CustomProxy =
    when {
        hasShadowsocks() -> shadowsocks.toDomain()
        hasSocks5Remote() -> socks5Remote.toDomain()
        hasSocks5Local() -> error("Socks5 local not supported")
        else -> error("Custom proxy not found")
    }

internal fun ManagementInterface.Shadowsocks.toDomain(): ApiAccessMethod.CustomProxy.Shadowsocks =
    ApiAccessMethod.CustomProxy.Shadowsocks(
        ip = ip,
        port = Port(port),
        password = password,
        cipher = Cipher.fromString(cipher),
    )

internal fun ManagementInterface.Socks5Remote.toDomain(): ApiAccessMethod.CustomProxy.Socks5Remote =
    ApiAccessMethod.CustomProxy.Socks5Remote(
        ip = ip,
        port = Port(port),
        auth =
            if (hasAuth()) {
                auth.toDomain()
            } else {
                null
            },
    )

internal fun ManagementInterface.SocksAuth.toDomain(): SocksAuth =
    SocksAuth(username = username, password = password)

internal fun ManagementInterface.FeatureIndicators.toDomain(): List<FeatureIndicator> =
    activeFeaturesList.map { it.toDomain() }.sorted()

internal fun ManagementInterface.FeatureIndicator.toDomain() =
    when (this) {
        ManagementInterface.FeatureIndicator.QUANTUM_RESISTANCE ->
            FeatureIndicator.QUANTUM_RESISTANCE
        ManagementInterface.FeatureIndicator.SPLIT_TUNNELING -> FeatureIndicator.SPLIT_TUNNELING
        ManagementInterface.FeatureIndicator.UDP_2_TCP -> FeatureIndicator.UDP_2_TCP
        ManagementInterface.FeatureIndicator.LAN_SHARING -> FeatureIndicator.LAN_SHARING
        ManagementInterface.FeatureIndicator.DNS_CONTENT_BLOCKERS ->
            FeatureIndicator.DNS_CONTENT_BLOCKERS
        ManagementInterface.FeatureIndicator.CUSTOM_DNS -> FeatureIndicator.CUSTOM_DNS
        ManagementInterface.FeatureIndicator.SERVER_IP_OVERRIDE ->
            FeatureIndicator.SERVER_IP_OVERRIDE
        ManagementInterface.FeatureIndicator.CUSTOM_MTU -> FeatureIndicator.CUSTOM_MTU
        ManagementInterface.FeatureIndicator.DAITA -> FeatureIndicator.DAITA
        ManagementInterface.FeatureIndicator.SHADOWSOCKS -> FeatureIndicator.SHADOWSOCKS
        ManagementInterface.FeatureIndicator.MULTIHOP -> FeatureIndicator.MULTIHOP
        ManagementInterface.FeatureIndicator.LOCKDOWN_MODE,
        ManagementInterface.FeatureIndicator.BRIDGE_MODE,
        ManagementInterface.FeatureIndicator.CUSTOM_MSS_FIX,
        ManagementInterface.FeatureIndicator.UNRECOGNIZED ->
            error("Feature not supported ${this.name}")
    }

internal fun ManagementInterface.IpVersion.toDomain() =
    when (this) {
        ManagementInterface.IpVersion.V4 -> IpVersion.IPV4
        ManagementInterface.IpVersion.V6 -> IpVersion.IPV6
        ManagementInterface.IpVersion.UNRECOGNIZED -> error("Not supported ${this.name}")
    }
