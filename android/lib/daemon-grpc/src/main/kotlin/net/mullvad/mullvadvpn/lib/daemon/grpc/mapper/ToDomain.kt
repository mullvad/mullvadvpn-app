@file:Suppress("TooManyFunctions")

package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import io.grpc.ConnectivityState
import java.net.InetAddress
import java.net.InetSocketAddress
import java.util.UUID
import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.lib.daemon.grpc.GrpcConnectivityState
import net.mullvad.mullvadvpn.lib.daemon.grpc.RelayNameComparator
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountId
import net.mullvad.mullvadvpn.lib.model.AccountToken
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.model.AppVersionInfo
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.Endpoint
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationEndpoint
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.ObfuscationType
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ParameterGenerationError
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.Provider
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
import net.mullvad.mullvadvpn.lib.model.SelectedObfuscation
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.SplitTunnelSettings
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint
import net.mullvad.mullvadvpn.lib.model.TunnelOptions
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData
import net.mullvad.mullvadvpn.lib.model.WireguardTunnelOptions
import org.joda.time.Instant

internal fun ManagementInterface.TunnelState.toDomain(): TunnelState =
    when (stateCase!!) {
        ManagementInterface.TunnelState.StateCase.DISCONNECTED ->
            TunnelState.Disconnected(
                location =
                    with(disconnected) {
                        if (hasDisconnectedLocation()) {
                            disconnectedLocation.toDomain()
                        } else null
                    },
            )
        ManagementInterface.TunnelState.StateCase.CONNECTING ->
            TunnelState.Connecting(
                endpoint = connecting.relayInfo.tunnelEndpoint.toDomain(),
                location =
                    with(connecting.relayInfo) {
                        if (hasLocation()) {
                            location.toDomain()
                        } else null
                    }
            )
        ManagementInterface.TunnelState.StateCase.CONNECTED ->
            TunnelState.Connected(
                endpoint = connected.relayInfo.tunnelEndpoint.toDomain(),
                location =
                    with(connected.relayInfo) {
                        if (hasLocation()) {
                            location.toDomain()
                        } else {
                            null
                        }
                    }
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
        city = city,
        latitude = latitude,
        longitude = longitude,
        hostname = hostname
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
                    protocol = protocol.toDomain()
                )
            },
        quantumResistant = quantumResistant,
        obfuscation =
            if (hasObfuscation()) {
                obfuscation.toDomain()
            } else {
                null
            }
    )

internal fun ManagementInterface.ObfuscationEndpoint.toDomain(): ObfuscationEndpoint =
    ObfuscationEndpoint(
        endpoint =
            Endpoint(address = InetSocketAddress(address, port), protocol = protocol.toDomain()),
        obfuscationType = obfuscationType.toDomain()
    )

internal fun ManagementInterface.ObfuscationType.toDomain(): ObfuscationType =
    when (this) {
        ManagementInterface.ObfuscationType.UDP2TCP -> ObfuscationType.Udp2Tcp
        ManagementInterface.ObfuscationType.UNRECOGNIZED ->
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

internal fun ManagementInterface.ErrorState.toDomain(): ErrorState =
    ErrorState(
        cause =
            when (cause!!) {
                ManagementInterface.ErrorState.Cause.AUTH_FAILED ->
                    ErrorStateCause.AuthFailed(authFailedError.name)
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
                ManagementInterface.ErrorState.Cause.VPN_PERMISSION_DENIED ->
                    ErrorStateCause.VpnPermissionDenied
                ManagementInterface.ErrorState.Cause.SPLIT_TUNNEL_ERROR ->
                    ErrorStateCause.StartTunnelError
                ManagementInterface.ErrorState.Cause.UNRECOGNIZED,
                ManagementInterface.ErrorState.Cause.CREATE_TUNNEL_DEVICE ->
                    throw IllegalArgumentException("Unrecognized error state cause")
            },
        isBlocking = !hasBlockingError()
    )

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
        autoConnect = autoConnect,
        tunnelOptions = tunnelOptions.toDomain(),
        relayOverrides = relayOverridesList.map { it.toDomain() },
        showBetaReleases = showBetaReleases,
        splitTunnelSettings = splitTunnel.toDomain()
    )

internal fun ManagementInterface.RelayOverride.toDomain(): RelayOverride =
    RelayOverride(
        hostname = hostname,
        ipv4AddressIn = if (hasIpv4AddrIn()) InetAddress.getByName(ipv4AddrIn) else null,
        ipv6AddressIn = if (hasIpv6AddrIn()) InetAddress.getByName(ipv6AddrIn) else null
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
        wireguardConstraints = wireguardConstraints.toDomain()
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
    if (isEmpty()) Constraint.Any else Constraint.Only(Providers(map { ProviderId(it) }.toSet()))

internal fun ManagementInterface.WireguardConstraints.toDomain(): WireguardConstraints =
    WireguardConstraints(
        port =
            if (hasPort()) {
                Constraint.Only(Port(port))
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
        selectedObfuscation = selectedObfuscation.toDomain(),
        udp2tcp = udp2Tcp.toDomain()
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
    if (hasPort()) {
        Udp2TcpObfuscationSettings(Constraint.Only(Port(port)))
    } else {
        Udp2TcpObfuscationSettings(Constraint.Any)
    }

internal fun ManagementInterface.CustomList.toDomain(): CustomList =
    CustomList(
        id = CustomListId(id),
        name = CustomListName.fromString(name),
        locations = locationsList.map { it.toDomain() }
    )

internal fun ManagementInterface.TunnelOptions.toDomain(): TunnelOptions =
    TunnelOptions(wireguard = wireguard.toDomain(), dnsOptions = dnsOptions.toDomain())

internal fun ManagementInterface.TunnelOptions.WireguardOptions.toDomain(): WireguardTunnelOptions =
    WireguardTunnelOptions(
        mtu = if (hasMtu()) Mtu(mtu) else null,
        quantumResistant = quantumResistant.toDomain(),
    )

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

internal fun SelectedObfuscation.toDomain():
    ManagementInterface.ObfuscationSettings.SelectedObfuscation =
    when (this) {
        SelectedObfuscation.Udp2Tcp ->
            ManagementInterface.ObfuscationSettings.SelectedObfuscation.UDP2TCP
        SelectedObfuscation.Auto -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.AUTO
        SelectedObfuscation.Off -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.OFF
    }

internal fun Udp2TcpObfuscationSettings.toDomain(): ManagementInterface.Udp2TcpObfuscationSettings =
    when (val port = port) {
        is Constraint.Any ->
            ManagementInterface.Udp2TcpObfuscationSettings.newBuilder().clearPort().build()
        is Constraint.Only ->
            ManagementInterface.Udp2TcpObfuscationSettings.newBuilder()
                .setPort(port.value.value)
                .build()
    }

internal fun ManagementInterface.AppVersionInfo.toDomain(): AppVersionInfo =
    AppVersionInfo(
        supported = supported,
        suggestedUpgrade = if (hasSuggestedUpgrade()) suggestedUpgrade else null
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
    WireguardEndpointData(portRangesList.map { it.toDomain() })

internal fun ManagementInterface.PortRange.toDomain(): PortRange = PortRange(first..last)

/**
 * Convert from a list of ManagementInterface.RelayListCountry to a model.RelayList. Non-wireguard
 * relays are filtered out. So are also cities that only contains non-wireguard relays and countries
 * that does not have any cities. Countries, cities and relays are ordered by name.
 */
@Suppress("LongMethod")
internal fun List<ManagementInterface.RelayListCountry>.toDomain():
    List<RelayItem.Location.Country> {
    val relayCountries =
        this.map { country ->
                val cities = mutableListOf<RelayItem.Location.City>()
                val relayCountry =
                    RelayItem.Location.Country(
                        GeoLocationId.Country(country.code),
                        country.name,
                        false,
                        cities
                    )

                for (city in country.citiesList) {
                    val relays = mutableListOf<RelayItem.Location.Relay>()
                    val relayCity =
                        RelayItem.Location.City(
                            name = city.name,
                            id = GeoLocationId.City(GeoLocationId.Country(country.code), city.code),
                            expanded = false,
                            relays = relays
                        )

                    val validCityRelays =
                        city.relaysList.filter {
                            it.endpointType == ManagementInterface.Relay.RelayType.WIREGUARD
                        }

                    for (relay in validCityRelays) {
                        relays.add(
                            RelayItem.Location.Relay(
                                id =
                                    GeoLocationId.Hostname(
                                        GeoLocationId.City(
                                            GeoLocationId.Country(country.code),
                                            city.code
                                        ),
                                        relay.hostname
                                    ),
                                active = relay.active,
                                provider =
                                    Provider(
                                        ProviderId(relay.provider),
                                        ownership =
                                            if (relay.owned) {
                                                Ownership.MullvadOwned
                                            } else {
                                                Ownership.Rented
                                            }
                                    )
                            )
                        )
                    }
                    relays.sortWith(RelayNameComparator)

                    if (relays.isNotEmpty()) {
                        cities.add(relayCity)
                    }
                }

                cities.sortBy { it.name }
                relayCountry
            }
            .filter { country -> country.cities.isNotEmpty() }
            .toMutableList()

    relayCountries.sortBy { it.name }

    return relayCountries.toList()
}

internal fun ManagementInterface.Device.toDomain(): Device =
    Device(DeviceId.fromString(id), name, Instant.ofEpochSecond(created.seconds).toDateTime())

internal fun ManagementInterface.DeviceState.toDomain(): DeviceState =
    when (state) {
        ManagementInterface.DeviceState.State.LOGGED_IN ->
            DeviceState.LoggedIn(AccountToken(device.accountToken), device.device.toDomain())
        ManagementInterface.DeviceState.State.LOGGED_OUT -> DeviceState.LoggedOut
        ManagementInterface.DeviceState.State.REVOKED -> DeviceState.Revoked
        ManagementInterface.DeviceState.State.UNRECOGNIZED ->
            throw IllegalArgumentException("Non valid device state")
        else -> throw NullPointerException("Device state is null")
    }

internal fun ManagementInterface.AccountData.toDomain(): AccountData =
    AccountData(
        AccountId(UUID.fromString(id)),
        expiryDate = Instant.ofEpochSecond(expiry.seconds).toDateTime()
    )

internal fun ManagementInterface.VoucherSubmission.toDomain(): RedeemVoucherSuccess =
    RedeemVoucherSuccess(
        timeAdded = secondsAdded,
        newExpiryDate = Instant.ofEpochSecond(newExpiry.seconds).toDateTime()
    )

internal fun ManagementInterface.SplitTunnelSettings.toDomain(): SplitTunnelSettings =
    SplitTunnelSettings(
        enabled = enableExclusions,
        excludedApps = appsList.map { AppId(it) }.toSet()
    )

internal fun ManagementInterface.PlayPurchasePaymentToken.toDomain(): PlayPurchasePaymentToken =
    PlayPurchasePaymentToken(value = token)

internal fun String.toDomain(): WebsiteAuthToken = WebsiteAuthToken(this)
