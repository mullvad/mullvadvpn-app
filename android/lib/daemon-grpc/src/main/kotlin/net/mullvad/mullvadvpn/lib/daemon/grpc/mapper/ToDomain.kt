@file:Suppress("TooManyFunctions")

package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import java.net.InetAddress
import java.net.InetSocketAddress
import java.time.Instant
import java.time.ZoneId
import java.util.UUID
import kotlin.collections.sorted
import mullvad_daemon.management_interface.AccessMethod
import mullvad_daemon.management_interface.AccessMethodSetting
import mullvad_daemon.management_interface.AccountData
import mullvad_daemon.management_interface.AfterDisconnect
import mullvad_daemon.management_interface.ApiAccessMethodSettings
import mullvad_daemon.management_interface.AppVersionInfo
import mullvad_daemon.management_interface.CustomDnsOptions
import mullvad_daemon.management_interface.CustomList
import mullvad_daemon.management_interface.CustomProxy
import mullvad_daemon.management_interface.DaitaSettings
import mullvad_daemon.management_interface.DefaultDnsOptions
import mullvad_daemon.management_interface.Device
import mullvad_daemon.management_interface.DeviceState
import mullvad_daemon.management_interface.DnsOptions
import mullvad_daemon.management_interface.ErrorState
import mullvad_daemon.management_interface.FeatureIndicator
import mullvad_daemon.management_interface.FeatureIndicators
import mullvad_daemon.management_interface.GeoIpLocation
import mullvad_daemon.management_interface.GeographicLocationConstraint
import mullvad_daemon.management_interface.IpVersion
import mullvad_daemon.management_interface.LocationConstraint
import mullvad_daemon.management_interface.NormalRelaySettings
import mullvad_daemon.management_interface.ObfuscationEndpoint
import mullvad_daemon.management_interface.ObfuscationSettings
import mullvad_daemon.management_interface.Ownership
import mullvad_daemon.management_interface.PlayPurchasePaymentToken
import mullvad_daemon.management_interface.PortRange
import mullvad_daemon.management_interface.QuantumResistantState
import mullvad_daemon.management_interface.Recent
import mullvad_daemon.management_interface.Recents
import mullvad_daemon.management_interface.Relay
import mullvad_daemon.management_interface.RelayList
import mullvad_daemon.management_interface.RelayListCity
import mullvad_daemon.management_interface.RelayListCountry
import mullvad_daemon.management_interface.RelayOverride
import mullvad_daemon.management_interface.RelaySettings
import mullvad_daemon.management_interface.Settings
import mullvad_daemon.management_interface.Shadowsocks
import mullvad_daemon.management_interface.Socks5Remote
import mullvad_daemon.management_interface.SocksAuth
import mullvad_daemon.management_interface.SplitTunnelSettings
import mullvad_daemon.management_interface.TransportProtocol
import mullvad_daemon.management_interface.TunnelEndpoint
import mullvad_daemon.management_interface.TunnelOptions
import mullvad_daemon.management_interface.TunnelState
import mullvad_daemon.management_interface.VoucherSubmission
import mullvad_daemon.management_interface.WireguardConstraints
import mullvad_daemon.management_interface.WireguardEndpointData
import net.mullvad.mullvadvpn.lib.daemon.grpc.RelayNameComparator
import net.mullvad.mullvadvpn.lib.model.AccountData as ModelAccountData
import net.mullvad.mullvadvpn.lib.model.AccountId
import net.mullvad.mullvadvpn.lib.model.AccountNumber as ModelAccountNumber
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect as ModelActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod as ModelApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId as ModelApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName as ModelApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting as ModelApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.model.AppVersionInfo as ModelAppVersionInfo
import net.mullvad.mullvadvpn.lib.model.AuthFailedError as ModelAuthFailedError
import net.mullvad.mullvadvpn.lib.model.Cipher as ModelCipher
import net.mullvad.mullvadvpn.lib.model.Constraint as ModelConstraint
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions as ModelCustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId as ModelCustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName as ModelCustomListName
import net.mullvad.mullvadvpn.lib.model.DaitaSettings as ModelDaitaSettings
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions as ModelDefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Device as ModelDevice
import net.mullvad.mullvadvpn.lib.model.DeviceId as ModelDeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState as ModelDeviceState
import net.mullvad.mullvadvpn.lib.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.lib.model.Endpoint as ModelEndpoint
import net.mullvad.mullvadvpn.lib.model.ErrorState as ModelErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause as ModelErrorStateCause
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator as ModelFeatureIndicator
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation as ModelGeoIpLocation
import net.mullvad.mullvadvpn.lib.model.GeoLocationId as ModelGeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion as ModelIpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu as ModelMtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationEndpoint as ModelObfuscationEndpoint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode as ModelObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings as ModelObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.ObfuscationType as ModelObfuscationType
import net.mullvad.mullvadvpn.lib.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError as ModelParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken as ModelPlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Port as ModelPort
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange as ModelPortRange
import net.mullvad.mullvadvpn.lib.model.ProviderId as ModelProviderId
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers as ModelProviders
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.lib.model.Quic
import net.mullvad.mullvadvpn.lib.model.Recent as ModelRecent
import net.mullvad.mullvadvpn.lib.model.Recents as ModelRecents
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherSuccess as ModelRedeemVoucherSuccess
import net.mullvad.mullvadvpn.lib.model.RelayConstraints as ModelRelayConstraints
import net.mullvad.mullvadvpn.lib.model.RelayItem as ModelRelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId as ModelRelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayList as ModelRelayList
import net.mullvad.mullvadvpn.lib.model.RelayOverride as ModelRelayOverride
import net.mullvad.mullvadvpn.lib.model.RelaySettings as ModelRelaySettings
import net.mullvad.mullvadvpn.lib.model.Settings as ModelSettings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksObfuscationSettings as ModelShadowsocksObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.SocksAuth as ModelSocksAuth
import net.mullvad.mullvadvpn.lib.model.SplitTunnelSettings as ModelSplitTunnelSettings
import net.mullvad.mullvadvpn.lib.model.TransportProtocol as ModelTransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint as ModelTunnelEndpoint
import net.mullvad.mullvadvpn.lib.model.TunnelOptions as ModelTunnelOptions
import net.mullvad.mullvadvpn.lib.model.TunnelState as ModelTunnelState
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings as ModelUdp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints as ModelWireguardConstraints
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData as ModelWireguardEndpointData

internal fun TunnelState.toDomain(): ModelTunnelState =
    when {
        disconnected != null -> disconnected.toDomain()
        connecting != null -> connecting.toDomain()
        connected != null -> connected.toDomain()
        disconnecting != null -> disconnecting.toDomain()
        error != null -> error.toDomain()
        else -> error("Tunnelstate not supported")
    }

private fun TunnelState.Connecting.toDomain(): ModelTunnelState.Connecting =
    ModelTunnelState.Connecting(
        endpoint = relay_info?.tunnel_endpoint?.toDomain(),
        location = relay_info?.location?.toDomain(),
        featureIndicators = feature_indicators?.toDomain() ?: emptyList(),
    )

private fun TunnelState.Disconnected.toDomain(): ModelTunnelState.Disconnected =
    ModelTunnelState.Disconnected(location = disconnected_location?.toDomain())

private fun TunnelState.Connected.toDomain(): ModelTunnelState.Connected =
    ModelTunnelState.Connected(
        endpoint = relay_info!!.tunnel_endpoint!!.toDomain(),
        location = relay_info.location?.toDomain(),
        featureIndicators = feature_indicators?.toDomain() ?: emptyList(),
    )

private fun TunnelState.Disconnecting.toDomain(): ModelTunnelState.Disconnecting =
    ModelTunnelState.Disconnecting(actionAfterDisconnect = after_disconnect.toDomain())

private fun TunnelState.Error.toDomain(): ModelTunnelState.Error {
    val otherAlwaysOnAppError =
        error_state?.let {
            if (it.other_always_on_app_error != null) {
                ModelErrorStateCause.OtherAlwaysOnApp(it.other_always_on_app_error.app_name)
            } else {
                null
            }
        }

    val invalidDnsServers =
        error_state?.let {
            if (it.invalid_dns_servers_error != null) {
                ModelErrorStateCause.InvalidDnsServers(
                    it.invalid_dns_servers_error.ip_addrs.map { address ->
                        InetAddress.getByName(address)
                    }
                )
            } else {
                null
            }
        }

    return ModelTunnelState.Error(
        errorState =
            error_state!!.toDomain(
                otherAlwaysOnApp = otherAlwaysOnAppError,
                invalidDnsServers = invalidDnsServers,
            )
    )
}

internal fun GeoIpLocation.toDomain(): ModelGeoIpLocation =
    ModelGeoIpLocation(
        ipv4 =
            if (ipv4 != null) {
                InetAddress.getByName(ipv4)
            } else {
                null
            },
        ipv6 =
            if (ipv6 != null) {
                InetAddress.getByName(ipv6)
            } else {
                null
            },
        country = country,
        city = city,
        latitude = latitude,
        longitude = longitude,
        hostname = hostname,
        entryHostname = entry_hostname,
    )

internal fun TunnelEndpoint.toDomain(): ModelTunnelEndpoint =
    ModelTunnelEndpoint(
        endpoint =
            ModelEndpoint(address = address.toInetSocketAddress(), protocol = protocol.toDomain()),
        entryEndpoint =
            if (entry_endpoint != null) {
                ModelEndpoint(
                    address = entry_endpoint.address.toInetSocketAddress(),
                    protocol = entry_endpoint.protocol.toDomain(),
                )
            } else {
                null
            },
        quantumResistant = quantum_resistant,
        obfuscation =
            if (obfuscation != null && obfuscation.single != null) {
                obfuscation.single.toDomain()
            } else {
                null
            },
        daita = daita,
    )

internal fun ObfuscationEndpoint.toDomain(): ModelObfuscationEndpoint =
    ModelObfuscationEndpoint(
        endpoint =
            ModelEndpoint(
                address = endpoint!!.address.toInetSocketAddress(),
                protocol = endpoint.protocol.toDomain(),
            ),
        obfuscationType = obfuscation_type.toDomain(),
    )

private fun String.toInetAddress(): InetAddress = InetAddress.getByName(this)

private fun String.toInetSocketAddress(): InetSocketAddress {
    val indexOfSeparator = indexOfLast { it == ':' }
    val ipPart = substring(0, indexOfSeparator).filter { it !in listOf('[', ']') }
    val portPart = substring(indexOfSeparator + 1)
    return InetSocketAddress(InetAddress.getByName(ipPart), portPart.toInt())
}

internal fun ObfuscationEndpoint.ObfuscationType.toDomain(): ModelObfuscationType =
    when (this) {
        ObfuscationEndpoint.ObfuscationType.UDP2TCP -> ModelObfuscationType.Udp2Tcp
        ObfuscationEndpoint.ObfuscationType.SHADOWSOCKS -> ModelObfuscationType.Shadowsocks
        ObfuscationEndpoint.ObfuscationType.QUIC -> ModelObfuscationType.Quic
        ObfuscationEndpoint.ObfuscationType.LWO -> ModelObfuscationType.Lwo
    }

internal fun TransportProtocol.toDomain(): ModelTransportProtocol =
    when (this) {
        TransportProtocol.TCP -> ModelTransportProtocol.Tcp
        TransportProtocol.UDP -> ModelTransportProtocol.Udp
    }

internal fun AfterDisconnect.toDomain(): ModelActionAfterDisconnect =
    when (this) {
        AfterDisconnect.NOTHING -> ModelActionAfterDisconnect.Nothing
        AfterDisconnect.RECONNECT -> ModelActionAfterDisconnect.Reconnect
        AfterDisconnect.BLOCK -> ModelActionAfterDisconnect.Block
    }

internal fun ErrorState.toDomain(
    otherAlwaysOnApp: ModelErrorStateCause.OtherAlwaysOnApp?,
    invalidDnsServers: ModelErrorStateCause.InvalidDnsServers?,
): ModelErrorState =
    ModelErrorState(
        cause =
            when (cause) {
                ErrorState.Cause.AUTH_FAILED ->
                    ModelErrorStateCause.AuthFailed(auth_failed_error.toDomain())
                ErrorState.Cause.IPV6_UNAVAILABLE -> ModelErrorStateCause.Ipv6Unavailable
                ErrorState.Cause.SET_FIREWALL_POLICY_ERROR if policy_error != null ->
                    policy_error.toDomain()
                ErrorState.Cause.SET_DNS_ERROR -> ModelErrorStateCause.DnsError
                ErrorState.Cause.START_TUNNEL_ERROR -> ModelErrorStateCause.StartTunnelError
                ErrorState.Cause.TUNNEL_PARAMETER_ERROR ->
                    ModelErrorStateCause.TunnelParameterError(parameter_error.toDomain())
                ErrorState.Cause.IS_OFFLINE -> ModelErrorStateCause.IsOffline
                ErrorState.Cause.SPLIT_TUNNEL_ERROR -> ModelErrorStateCause.StartTunnelError
                ErrorState.Cause.NEED_FULL_DISK_PERMISSIONS,
                ErrorState.Cause.CREATE_TUNNEL_DEVICE ->
                    throw IllegalArgumentException("Unrecognized error state cause")
                ErrorState.Cause.NOT_PREPARED -> ModelErrorStateCause.NotPrepared
                ErrorState.Cause.OTHER_ALWAYS_ON_APP -> otherAlwaysOnApp!!
                ErrorState.Cause.OTHER_LEGACY_ALWAYS_ON_VPN ->
                    ModelErrorStateCause.OtherLegacyAlwaysOnApp
                ErrorState.Cause.INVALID_DNS_SERVERS -> invalidDnsServers!!
                else -> error("Invalid error state cause $cause")
            },
        isBlocking = blocking_error != null,
    )

private fun ErrorState.AuthFailedError.toDomain(): ModelAuthFailedError =
    when (this) {
        ErrorState.AuthFailedError.UNKNOWN -> ModelAuthFailedError.Unknown
        ErrorState.AuthFailedError.INVALID_ACCOUNT -> ModelAuthFailedError.InvalidAccount
        ErrorState.AuthFailedError.EXPIRED_ACCOUNT -> ModelAuthFailedError.ExpiredAccount
        ErrorState.AuthFailedError.TOO_MANY_CONNECTIONS -> ModelAuthFailedError.TooManyConnections
    }

internal fun ErrorState.FirewallPolicyError.toDomain(): ModelErrorStateCause.FirewallPolicyError =
    when (type) {
        ErrorState.FirewallPolicyError.ErrorType.GENERIC ->
            ModelErrorStateCause.FirewallPolicyError.Generic
        ErrorState.FirewallPolicyError.ErrorType.LOCKED ->
            throw IllegalArgumentException("Unrecognized firewall policy error")
    }

internal fun ErrorState.GenerationError.toDomain(): ModelParameterGenerationError =
    when (this) {
        ErrorState.GenerationError.NO_MATCHING_RELAY_ENTRY ->
            ModelParameterGenerationError.NoMatchingRelayEntry
        ErrorState.GenerationError.NO_MATCHING_RELAY_EXIT ->
            ModelParameterGenerationError.NoMatchingRelayExit
        ErrorState.GenerationError.NO_MATCHING_RELAY ->
            ModelParameterGenerationError.NoMatchingRelay
        ErrorState.GenerationError.NO_MATCHING_BRIDGE_RELAY ->
            ModelParameterGenerationError.NoMatchingBridgeRelay
        ErrorState.GenerationError.CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR ->
            ModelParameterGenerationError.CustomTunnelHostResolutionError
        ErrorState.GenerationError.NETWORK_IPV4_UNAVAILABLE ->
            ModelParameterGenerationError.Ipv4_Unavailable
        ErrorState.GenerationError.NETWORK_IPV6_UNAVAILABLE ->
            ModelParameterGenerationError.Ipv6_Unavailable
    }

internal fun Settings.toDomain(): ModelSettings =
    ModelSettings(
        relaySettings = relay_settings!!.toDomain(),
        obfuscationSettings = obfuscation_settings!!.toDomain(),
        customLists = custom_lists?.custom_lists?.map { it.toDomain() } ?: emptyList(),
        allowLan = allow_lan,
        tunnelOptions = tunnel_options!!.toDomain(),
        relayOverrides = relay_overrides.map { it.toDomain() },
        showBetaReleases = show_beta_releases,
        splitTunnelSettings = split_tunnel!!.toDomain(),
        apiAccessMethodSettings = api_access_methods!!.toDomain(),
        recents = recents.toDomain(),
    )

internal fun RelayOverride.toDomain(): ModelRelayOverride =
    ModelRelayOverride(
        hostname = hostname,
        ipv4AddressIn = if (ipv4_addr_in != null) InetAddress.getByName(ipv4_addr_in) else null,
        ipv6AddressIn = if (ipv6_addr_in != null) InetAddress.getByName(ipv6_addr_in) else null,
    )

internal fun RelaySettings.toDomain(): ModelRelaySettings =
    when {
        custom != null -> throw IllegalArgumentException("CustomTunnelEndpoint is not supported")
        normal != null -> ModelRelaySettings(normal.toDomain())
        else -> throw NullPointerException("RelaySettings endpoint is null")
    }

internal fun NormalRelaySettings.toDomain(): ModelRelayConstraints =
    ModelRelayConstraints(
        location = location?.toDomain() ?: ModelConstraint.Any,
        providers = providers.toDomain(),
        ownership = ownership.toDomain(),
        wireguardConstraints = wireguard_constraints!!.toDomain(),
    )

internal fun LocationConstraint.toDomain(): ModelConstraint<ModelRelayItemId> =
    when {
        custom_list != null -> ModelConstraint.Only(ModelCustomListId(custom_list))
        location != null -> ModelConstraint.Only(location.toDomain())
        else -> throw IllegalArgumentException("Invalid location constraint")
    }

@Suppress("ReturnCount")
internal fun GeographicLocationConstraint.toDomain(): ModelGeoLocationId {
    val country = ModelGeoLocationId.Country(country)
    if (city == null) {
        return country
    }

    val city = ModelGeoLocationId.City(country, city)
    if (hostname == null) {
        return city
    }
    return ModelGeoLocationId.Hostname(city, hostname)
}

internal fun List<String>.toDomain(): ModelConstraint<ModelProviders> =
    if (isEmpty()) ModelConstraint.Any
    else ModelConstraint.Only(map { ModelProviderId(it) }.toSet())

internal fun WireguardConstraints.toDomain(): ModelWireguardConstraints =
    ModelWireguardConstraints(
        isMultihopEnabled = use_multihop,
        entryLocation = entry_location?.toDomain() ?: ModelConstraint.Any,
        ipVersion =
            if (ip_version != null) {
                ModelConstraint.Only(ip_version.toDomain())
            } else {
                ModelConstraint.Any
            },
    )

internal fun Ownership.toDomain(): ModelConstraint<ModelOwnership> =
    when (this) {
        Ownership.ANY -> ModelConstraint.Any
        Ownership.MULLVAD_OWNED -> ModelConstraint.Only(ModelOwnership.MullvadOwned)
        Ownership.RENTED -> ModelConstraint.Only(ModelOwnership.Rented)
    }

internal fun ObfuscationSettings.toDomain(): ModelObfuscationSettings =
    ModelObfuscationSettings(
        selectedObfuscationMode = selected_obfuscation.toDomain(),
        udp2tcp = udp2tcp!!.toDomain(),
        shadowsocks = shadowsocks!!.toDomain(),
        wireguardPort = wireguard_port!!.toDomain(),
    )

internal fun ObfuscationSettings.SelectedObfuscation.toDomain(): ModelObfuscationMode =
    when (this) {
        ObfuscationSettings.SelectedObfuscation.AUTO -> ObfuscationMode.Auto
        ObfuscationSettings.SelectedObfuscation.OFF -> ObfuscationMode.Off
        ObfuscationSettings.SelectedObfuscation.UDP2TCP -> ObfuscationMode.Udp2Tcp
        ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS -> ObfuscationMode.Shadowsocks
        ObfuscationSettings.SelectedObfuscation.QUIC -> ObfuscationMode.Quic
        ObfuscationSettings.SelectedObfuscation.LWO -> ObfuscationMode.Lwo
        ObfuscationSettings.SelectedObfuscation.WIREGUARD_PORT -> ObfuscationMode.WireguardPort
    }

internal fun ObfuscationSettings.Udp2TcpObfuscation.toDomain(): ModelUdp2TcpObfuscationSettings =
    if (port != null) {
        ModelUdp2TcpObfuscationSettings(Constraint.Only(Port(port)))
    } else {
        ModelUdp2TcpObfuscationSettings(ModelConstraint.Any)
    }

internal fun ObfuscationSettings.Shadowsocks.toDomain(): ModelShadowsocksObfuscationSettings =
    if (port != null) {
        ModelShadowsocksObfuscationSettings(Constraint.Only(Port(port)))
    } else {
        ModelShadowsocksObfuscationSettings(Constraint.Any)
    }

internal fun ObfuscationSettings.WireguardPort.toDomain(): Constraint<Port> =
    if (port != null) {
        Constraint.Only(Port(port))
    } else {
        Constraint.Any
    }

internal fun CustomList.toDomain(): ModelCustomList =
    ModelCustomList(
        id = ModelCustomListId(id),
        name = ModelCustomListName.fromString(name),
        locations = locations.map { it.toDomain() },
    )

internal fun TunnelOptions.toDomain(): ModelTunnelOptions =
    ModelTunnelOptions(
        mtu = if (mtu != null) ModelMtu(mtu) else null,
        quantumResistant = quantum_resistant!!.toDomain(),
        daitaSettings = daita!!.toDomain(),
        dnsOptions = dns_options!!.toDomain(),
        enableIpv6 = enable_ipv6,
    )

internal fun DaitaSettings.toDomain(): ModelDaitaSettings =
    ModelDaitaSettings(enabled = enabled, directOnly = direct_only)

internal fun QuantumResistantState.toDomain(): ModelQuantumResistantState =
    when (state) {
        QuantumResistantState.State.ON -> ModelQuantumResistantState.On
        QuantumResistantState.State.OFF -> ModelQuantumResistantState.Off
    }

internal fun DnsOptions.toDomain(): ModelDnsOptions =
    ModelDnsOptions(
        state = state.toDomain(),
        defaultOptions = default_options!!.toDomain(),
        customOptions = custom_options!!.toDomain(),
    )

internal fun DnsOptions.DnsState.toDomain(): ModelDnsState =
    when (this) {
        DnsOptions.DnsState.DEFAULT -> ModelDnsState.Default
        DnsOptions.DnsState.CUSTOM -> ModelDnsState.Custom
    }

internal fun DefaultDnsOptions.toDomain() =
    ModelDefaultDnsOptions(
        blockAds = block_ads,
        blockMalware = block_malware,
        blockAdultContent = block_adult_content,
        blockGambling = block_gambling,
        blockSocialMedia = block_social_media,
        blockTrackers = block_trackers,
    )

internal fun CustomDnsOptions.toDomain() =
    ModelCustomDnsOptions(addresses.map { InetAddress.getByName(it) })

internal fun AppVersionInfo.toDomain(): ModelAppVersionInfo =
    ModelAppVersionInfo(supported = supported, suggestedUpgrade = suggested_upgrade?.version)

/*internal fun ConnectivityState.toDomain(): GrpcConnectivityState =
when (this) {
    ConnectivityState.CONNECTING -> GrpcConnectivityState.Connecting
    ConnectivityState.READY -> GrpcConnectivityState.Ready
    ConnectivityState.IDLE -> GrpcConnectivityState.Idle
    ConnectivityState.TRANSIENT_FAILURE -> GrpcConnectivityState.TransientFailure
    ConnectivityState.SHUTDOWN -> GrpcConnectivityState.Shutdown
}*/

internal fun RelayList.toDomain(): ModelRelayList =
    ModelRelayList(countries.toDomain(), endpoint_data!!.toDomain())

internal fun WireguardEndpointData.toDomain(): ModelWireguardEndpointData =
    ModelWireguardEndpointData(
        port_ranges.map { it.toDomain() },
        shadowsocks_port_ranges.map { it.toDomain() },
    )

internal fun PortRange.toDomain(): ModelPortRange = ModelPortRange(first..last)

/**
 * Convert from a list of RelayListCountry to a model.RelayList. Non-wireguard relays are filtered
 * out. So are also cities that only contains non-wireguard relays and countries that does not have
 * any cities. Countries, cities and relays are ordered by name.
 */
internal fun List<RelayListCountry>.toDomain(): List<ModelRelayItem.Location.Country> =
    map(RelayListCountry::toDomain).filter { it.cities.isNotEmpty() }.sortedBy { it.name }

internal fun RelayListCountry.toDomain(): ModelRelayItem.Location.Country {
    val countryCode = ModelGeoLocationId.Country(code)
    return ModelRelayItem.Location.Country(
        countryCode,
        name,
        cities
            .map { city -> city.toDomain(countryCode) }
            .filter { it.relays.isNotEmpty() }
            .sortedBy { it.name },
    )
}

internal fun RelayListCity.toDomain(
    countryCode: ModelGeoLocationId.Country
): ModelRelayItem.Location.City {
    val cityCode = ModelGeoLocationId.City(countryCode, code)
    return ModelRelayItem.Location.City(
        name = name,
        id = cityCode,
        relays = relays.map { it.toDomain(cityCode) }.sortedWith(RelayNameComparator),
    )
}

internal fun Relay.toDomain(cityCode: ModelGeoLocationId.City): ModelRelayItem.Location.Relay =
    ModelRelayItem.Location.Relay(
        id = ModelGeoLocationId.Hostname(cityCode, hostname),
        active = active,
        provider = ProviderId(provider),
        ownership = if (owned) ModelOwnership.MullvadOwned else ModelOwnership.Rented,
        daita = endpoint_data?.daita ?: false,
        quic = endpoint_data?.quic?.toDomain(),
        lwo = endpoint_data?.lwo ?: false,
    )

private fun Relay.WireguardEndpoint.Quic.toDomain(): Quic =
    Quic(inAddresses = addr_in.map { it.toInetAddress() })

private fun Instant.atDefaultZone() = atZone(ZoneId.systemDefault())

internal fun Device.toDomain(): ModelDevice =
    ModelDevice(ModelDeviceId.fromString(id), name, created!!.atDefaultZone())

internal fun DeviceState.toDomain(): ModelDeviceState =
    when (state) {
        DeviceState.State.LOGGED_IN if device != null && device.device != null ->
            ModelDeviceState.LoggedIn(
                ModelAccountNumber(device.account_number),
                device.device.toDomain(),
            )
        DeviceState.State.LOGGED_OUT -> ModelDeviceState.LoggedOut
        DeviceState.State.REVOKED -> ModelDeviceState.Revoked
        else -> throw NullPointerException("Device state is null")
    }

internal fun AccountData.toDomain(accountNumber: ModelAccountNumber): ModelAccountData =
    ModelAccountData(
        id = AccountId(UUID.fromString(id)),
        accountNumber = accountNumber,
        expiryDate = expiry!!.atDefaultZone(),
    )

internal fun VoucherSubmission.toDomain(): ModelRedeemVoucherSuccess =
    ModelRedeemVoucherSuccess(
        timeAdded = seconds_added,
        newExpiryDate = new_expiry!!.atDefaultZone(),
    )

internal fun SplitTunnelSettings.toDomain(): ModelSplitTunnelSettings =
    ModelSplitTunnelSettings(
        enabled = enable_exclusions,
        excludedApps = apps.map { AppId(it) }.toSet(),
    )

internal fun PlayPurchasePaymentToken.toDomain(): ModelPlayPurchasePaymentToken =
    ModelPlayPurchasePaymentToken(value = token)

internal fun ApiAccessMethodSettings.toDomain(): List<ModelApiAccessMethodSetting> = buildList {
    if (direct != null) {
        add(direct.toDomain())
    }
    if (mullvad_bridges != null) {
        add(mullvad_bridges.toDomain())
    }
    if (encrypted_dns_proxy != null) {
        add(encrypted_dns_proxy.toDomain())
    }
    addAll(custom.map { it.toDomain() })
}

internal fun AccessMethodSetting.toDomain(): ModelApiAccessMethodSetting =
    ModelApiAccessMethodSetting(
        id = ModelApiAccessMethodId.fromString(id!!.value_),
        name = ModelApiAccessMethodName.fromString(name),
        enabled = enabled,
        apiAccessMethod = access_method.toDomain(),
    )

internal fun AccessMethod?.toDomain(): ModelApiAccessMethod =
    when {
        this == null -> error("Access method is null")
        direct != null -> ModelApiAccessMethod.Direct
        bridges != null -> ModelApiAccessMethod.Bridges
        encrypted_dns_proxy != null -> ModelApiAccessMethod.EncryptedDns
        custom != null -> custom.toDomain()
        else -> error("Type not found")
    }

internal fun CustomProxy.toDomain(): ModelApiAccessMethod.CustomProxy =
    when {
        shadowsocks != null -> shadowsocks.toDomain()
        socks5remote != null -> socks5remote.toDomain()
        socks5local != null -> error("Socks5 local not supported")
        else -> error("Custom proxy not found")
    }

internal fun Shadowsocks.toDomain(): ModelApiAccessMethod.CustomProxy.Shadowsocks =
    ModelApiAccessMethod.CustomProxy.Shadowsocks(
        ip = ip,
        port = ModelPort(port),
        password = password,
        cipher = ModelCipher.fromString(cipher),
    )

internal fun Socks5Remote.toDomain(): ModelApiAccessMethod.CustomProxy.Socks5Remote =
    ModelApiAccessMethod.CustomProxy.Socks5Remote(
        ip = ip,
        port = ModelPort(port),
        auth = auth?.toDomain(),
    )

internal fun SocksAuth.toDomain(): ModelSocksAuth =
    ModelSocksAuth(username = username, password = password)

internal fun FeatureIndicators.toDomain(): List<ModelFeatureIndicator> =
    active_features.map { it.toDomain() }.sorted()

@Suppress("ComplexMethod")
internal fun FeatureIndicator.toDomain(): ModelFeatureIndicator =
    when (this) {
        FeatureIndicator.QUANTUM_RESISTANCE -> ModelFeatureIndicator.QUANTUM_RESISTANCE
        FeatureIndicator.SPLIT_TUNNELING -> ModelFeatureIndicator.SPLIT_TUNNELING
        FeatureIndicator.UDP_2_TCP -> ModelFeatureIndicator.UDP_2_TCP
        FeatureIndicator.LAN_SHARING -> ModelFeatureIndicator.LAN_SHARING
        FeatureIndicator.DNS_CONTENT_BLOCKERS -> ModelFeatureIndicator.DNS_CONTENT_BLOCKERS
        FeatureIndicator.CUSTOM_DNS -> ModelFeatureIndicator.CUSTOM_DNS
        FeatureIndicator.SERVER_IP_OVERRIDE -> ModelFeatureIndicator.SERVER_IP_OVERRIDE
        FeatureIndicator.CUSTOM_MTU -> ModelFeatureIndicator.CUSTOM_MTU
        FeatureIndicator.DAITA -> ModelFeatureIndicator.DAITA
        FeatureIndicator.SHADOWSOCKS -> ModelFeatureIndicator.SHADOWSOCKS
        FeatureIndicator.MULTIHOP -> ModelFeatureIndicator.MULTIHOP
        FeatureIndicator.DAITA_MULTIHOP -> ModelFeatureIndicator.DAITA_MULTIHOP
        FeatureIndicator.QUIC -> ModelFeatureIndicator.QUIC
        FeatureIndicator.LWO -> ModelFeatureIndicator.LWO
        FeatureIndicator.WIREGUARD_PORT -> ModelFeatureIndicator.WIREGUARD_PORT
        FeatureIndicator.LOCKDOWN_MODE -> error("Feature not supported ${this.name}")
    }

internal fun IpVersion.toDomain() =
    when (this) {
        IpVersion.V4 -> ModelIpVersion.IPV4
        IpVersion.V6 -> ModelIpVersion.IPV6
    }

internal fun Recents?.toDomain(): ModelRecents =
    if (this != null) {
        ModelRecents.Enabled(recents = recents.map { it.toDomain() })
    } else {
        ModelRecents.Disabled
    }

internal fun Recent.toDomain(): ModelRecent =
    when {
        multihop != null ->
            ModelRecent.Multihop(
                entry = (multihop.entry!!.toDomain() as ModelConstraint.Only).value,
                exit = (multihop.exit!!.toDomain() as ModelConstraint.Only).value,
            )

        singlehop != null ->
            ModelRecent.Singlehop((singlehop.toDomain() as ModelConstraint.Only).value)

        else -> error("Recent type must be set")
    }
