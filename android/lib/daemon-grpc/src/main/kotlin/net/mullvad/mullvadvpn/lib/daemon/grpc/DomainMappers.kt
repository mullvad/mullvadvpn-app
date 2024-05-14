package net.mullvad.mullvadvpn.lib.daemon.grpc

import android.net.Uri
import io.grpc.ConnectivityState
import java.net.InetAddress
import java.net.InetSocketAddress
import java.util.UUID
import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.AccountId
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.AppId
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.Endpoint
import net.mullvad.mullvadvpn.model.ErrorState
import net.mullvad.mullvadvpn.model.ErrorStateCause
import net.mullvad.mullvadvpn.model.FirewallPolicyError
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.Mtu
import net.mullvad.mullvadvpn.model.ObfuscationEndpoint
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.ObfuscationType
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.ParameterGenerationError
import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.ProviderId
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.RedeemVoucherSuccess
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.model.RelayItemId
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelayOverride
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.SplitTunnelSettings
import net.mullvad.mullvadvpn.model.TransportProtocol
import net.mullvad.mullvadvpn.model.TunnelEndpoint
import net.mullvad.mullvadvpn.model.TunnelOptions
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardEndpointData
import net.mullvad.mullvadvpn.model.WireguardTunnelOptions
import net.mullvad.mullvadvpn.model.WwwAuthToken
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
            with(address) {
                val indexOfSeparator = indexOfLast { it == ':' }
                val ipPart =
                    address.substring(0, indexOfSeparator).filter { it !in listOf('[', ']') }
                val portPart = address.substring(indexOfSeparator + 1)

                Endpoint(
                    address = InetSocketAddress(InetAddress.getByName(ipPart), portPart.toInt()),
                    protocol = this@toDomain.protocol.toDomain()
                )
            },
        quantumResistant = this.quantumResistant,
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
        isBlocking = !this.hasBlockingError()
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
        ManagementInterface.RelaySettings.EndpointCase.NORMAL ->
            RelaySettings(this.normal.toDomain())
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

internal fun Constraint<RelayItemId>.fromDomain(): ManagementInterface.LocationConstraint =
    ManagementInterface.LocationConstraint.newBuilder()
        .apply {
            when (this@fromDomain) {
                is Constraint.Any -> {}
                is Constraint.Only -> {
                    when (val relayItemId = value) {
                        is CustomListId -> setCustomList(relayItemId.value)
                        is GeoLocationId -> setLocation(relayItemId.fromDomain())
                    }
                }
            }
        }
        .build()

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
    if (isEmpty()) Constraint.Any
    else Constraint.Only(Providers(this.map { ProviderId(it) }.toSet()))

internal fun Constraint<Providers>.fromDomain(): List<String> =
    when (this) {
        is Constraint.Any -> emptyList()
        is Constraint.Only -> value.providers.map { it.value }
    }

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
        quantumResistant = this.quantumResistant.toDomain(),
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
        currentDnsOption = this.state.toDomain(),
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

internal fun DnsOptions.fromDomain(): ManagementInterface.DnsOptions =
    ManagementInterface.DnsOptions.newBuilder()
        .setState(this.currentDnsOption.fromDomain())
        .setCustomOptions(this.customOptions.fromDomain())
        .setDefaultOptions(this.defaultOptions.fromDomain())
        .build()

internal fun DnsState.fromDomain(): ManagementInterface.DnsOptions.DnsState =
    when (this) {
        DnsState.Default -> ManagementInterface.DnsOptions.DnsState.DEFAULT
        DnsState.Custom -> ManagementInterface.DnsOptions.DnsState.CUSTOM
    }

internal fun CustomDnsOptions.fromDomain(): ManagementInterface.CustomDnsOptions =
    ManagementInterface.CustomDnsOptions.newBuilder()
        .addAllAddresses(this.addresses.map { it.hostAddress })
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
    AppVersionInfo(
        supported = this.supported,
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

internal fun RelayItemId.fromDomain(): ManagementInterface.LocationConstraint =
    when (this) {
        is CustomListId ->
            ManagementInterface.LocationConstraint.newBuilder().setCustomList(this.value).build()
        is GeoLocationId ->
            ManagementInterface.LocationConstraint.newBuilder()
                .setLocation(this.fromDomain())
                .build()
    }

internal fun GeoLocationId.fromDomain(): ManagementInterface.GeographicLocationConstraint =
    ManagementInterface.GeographicLocationConstraint.newBuilder()
        .apply {
            when (val id = this@fromDomain) {
                is GeoLocationId.Country -> setCountry(id.countryCode)
                is GeoLocationId.City -> setCountry(id.countryCode.countryCode).setCity(id.cityCode)
                is GeoLocationId.Hostname ->
                    setCountry(id.country.countryCode)
                        .setCity(id.city.cityCode)
                        .setHostname(id.hostname)
            }
        }
        .build()

internal fun CustomList.fromDomain(): ManagementInterface.CustomList =
    ManagementInterface.CustomList.newBuilder()
        .setId(this.id.value)
        .setName(this.name.value)
        .addAllLocations(this.locations.map { it.fromDomain() })
        .build()

internal fun ManagementInterface.RelayList.toDomain(): RelayList =
    RelayList(countriesList.toDomain(), wireguard.toDomain())

internal fun ManagementInterface.WireguardEndpointData.toDomain(): WireguardEndpointData =
    WireguardEndpointData(portRangesList.map { it.toDomain() })

internal fun ManagementInterface.PortRange.toDomain(): PortRange = PortRange(first..last)

internal fun WireguardConstraints.fromDomain(): ManagementInterface.WireguardConstraints =
    when (this.port) {
        is Constraint.Any -> ManagementInterface.WireguardConstraints.newBuilder().build()
        is Constraint.Only ->
            ManagementInterface.WireguardConstraints.newBuilder()
                .setPort((this.port as Constraint.Only<Port>).value.value)
                .build()
    }

/**
 * Convert from a list of ManagementInterface.RelayListCountry to a model.RelayList. Non-wireguard
 * relays are filtered out. So are also cities that only contains non-wireguard relays and countries
 * that does not have any cities. Countries, cities and relays are ordered by name.
 */
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

internal fun Ownership.fromDomain(): ManagementInterface.Ownership =
    when (this) {
        Ownership.MullvadOwned -> ManagementInterface.Ownership.MULLVAD_OWNED
        Ownership.Rented -> ManagementInterface.Ownership.RENTED
    }

internal fun ManagementInterface.Device.toDomain(): Device =
    Device(
        DeviceId.fromString(id),
        name,
        pubkey.toByteArray(),
        Instant.ofEpochSecond(created.seconds).toDateTime()
    )

internal fun ManagementInterface.DeviceState.toDomain(): DeviceState =
    when (state) {
        ManagementInterface.DeviceState.State.LOGGED_IN ->
            DeviceState.LoggedIn(AccountToken(this.device.accountToken), device.device.toDomain())
        ManagementInterface.DeviceState.State.LOGGED_OUT -> DeviceState.LoggedOut
        ManagementInterface.DeviceState.State.REVOKED -> DeviceState.Revoked
        ManagementInterface.DeviceState.State.UNRECOGNIZED ->
            throw IllegalArgumentException("Non valid device state")
        else -> throw NullPointerException("Device state is null")
    }

internal fun RelaySettings.fromDomain(): ManagementInterface.RelaySettings =
    ManagementInterface.RelaySettings.newBuilder()
        .setNormal(
            ManagementInterface.NormalRelaySettings.newBuilder()
                .setTunnelType(ManagementInterface.TunnelType.WIREGUARD)
                .setWireguardConstraints(relayConstraints.wireguardConstraints.fromDomain())
                .setOpenvpnConstraints(ManagementInterface.OpenvpnConstraints.getDefaultInstance())
                .setLocation(this@fromDomain.relayConstraints.location.fromDomain())
                .setOwnership(relayConstraints.ownership.fromDomain())
                .addAllProviders(relayConstraints.providers.fromDomain())
                .build()
        )
        .build()

internal fun Constraint<Ownership>.fromDomain(): ManagementInterface.Ownership =
    when (this) {
        Constraint.Any -> ManagementInterface.Ownership.ANY
        is Constraint.Only -> value.fromDomain()
    }

internal fun ManagementInterface.AccountData.toDomain(): AccountData =
    AccountData(
        AccountId(UUID.fromString(id)),
        expiryDate = Instant.ofEpochSecond(expiry.seconds).toDateTime()
    )

internal fun ManagementInterface.VoucherSubmission.toDomain(): RedeemVoucherSuccess =
    RedeemVoucherSuccess(
        timeAdded = secondsAdded,
        newExpiry = Instant.ofEpochSecond(newExpiry.seconds).toDateTime()
    )

internal fun ManagementInterface.SplitTunnelSettings.toDomain(): SplitTunnelSettings =
    SplitTunnelSettings(
        enabled = enableExclusions,
        excludedApps = appsList.map { AppId(it) }.toSet()
    )

internal fun ManagementInterface.PlayPurchasePaymentToken.toDomain(): PlayPurchasePaymentToken =
    PlayPurchasePaymentToken(value = this.token)

internal fun PlayPurchasePaymentToken.fromDomain(): ManagementInterface.PlayPurchasePaymentToken =
    ManagementInterface.PlayPurchasePaymentToken.newBuilder()
        .setToken(
            value,
        )
        .build()

internal fun PlayPurchase.fromDomain(): ManagementInterface.PlayPurchase =
    ManagementInterface.PlayPurchase.newBuilder()
        .setPurchaseToken(purchaseToken.fromDomain())
        .setProductId(productId)
        .build()

internal fun String.toDomain(): WwwAuthToken = WwwAuthToken(this)
