package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import mullvad_daemon.management_interface.AccessMethod
import mullvad_daemon.management_interface.AccessMethodSetting
import mullvad_daemon.management_interface.CustomDnsOptions
import mullvad_daemon.management_interface.CustomList
import mullvad_daemon.management_interface.CustomProxy
import mullvad_daemon.management_interface.DaitaSettings
import mullvad_daemon.management_interface.DefaultDnsOptions
import mullvad_daemon.management_interface.DnsOptions
import mullvad_daemon.management_interface.GeographicLocationConstraint
import mullvad_daemon.management_interface.IpVersion
import mullvad_daemon.management_interface.LocationConstraint
import mullvad_daemon.management_interface.NewAccessMethodSetting
import mullvad_daemon.management_interface.NormalRelaySettings
import mullvad_daemon.management_interface.ObfuscationSettings
import mullvad_daemon.management_interface.Ownership
import mullvad_daemon.management_interface.PlayPurchase
import mullvad_daemon.management_interface.PlayPurchasePaymentToken
import mullvad_daemon.management_interface.QuantumResistantState
import mullvad_daemon.management_interface.RelaySettings
import mullvad_daemon.management_interface.Shadowsocks
import mullvad_daemon.management_interface.ShadowsocksSettings
import mullvad_daemon.management_interface.Socks5Remote
import mullvad_daemon.management_interface.SocksAuth
import mullvad_daemon.management_interface.TransportProtocol
import mullvad_daemon.management_interface.UUID
import mullvad_daemon.management_interface.Udp2TcpObfuscationSettings
import mullvad_daemon.management_interface.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions as ModelCustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.DaitaSettings as ModelDaitaSettings
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions as ModelDefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion as ModelIpVersion
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting as ModelNewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode as ModelObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings as ModelObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.lib.model.PlayPurchase as ModelPlayPurchase
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken as ModelPlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelaySettings as ModelRelaySettings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksSettings as ModelShadowsocksSettings
import net.mullvad.mullvadvpn.lib.model.SocksAuth as ModelSocksAuth
import net.mullvad.mullvadvpn.lib.model.TransportProtocol as ModelTransportProtocol
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings as ModelUdp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints as ModelWireguardConstraints

internal fun Constraint<RelayItemId>.fromDomain(): LocationConstraint =
    when (this@fromDomain) {
        is Constraint.Any -> LocationConstraint()
        is Constraint.Only -> {
            when (val relayItemId = value) {
                is CustomListId -> LocationConstraint(custom_list = relayItemId.value)
                is GeoLocationId -> LocationConstraint(location = relayItemId.fromDomain())
            }
        }
    }

internal fun Constraint<Providers>.fromDomain(): List<String> =
    when (this) {
        is Constraint.Any -> emptyList()
        is Constraint.Only -> value.map { it.value }
    }

internal fun ModelDnsOptions.fromDomain(): DnsOptions =
    DnsOptions(
        state = state.fromDomain(),
        custom_options = customOptions.fromDomain(),
        default_options = defaultOptions.fromDomain(),
    )

internal fun ModelDnsState.fromDomain(): DnsOptions.DnsState =
    when (this) {
        ModelDnsState.Default -> DnsOptions.DnsState.DEFAULT
        ModelDnsState.Custom -> DnsOptions.DnsState.CUSTOM
    }

internal fun ModelCustomDnsOptions.fromDomain(): CustomDnsOptions =
    CustomDnsOptions(addresses = addresses.mapNotNull { it.hostAddress })

internal fun ModelDefaultDnsOptions.fromDomain(): DefaultDnsOptions =
    DefaultDnsOptions(
        block_ads = blockAds,
        block_gambling = blockGambling,
        block_malware = blockMalware,
        block_trackers = blockTrackers,
        block_adult_content = blockAdultContent,
        block_social_media = blockSocialMedia,
    )

internal fun ModelObfuscationSettings.fromDomain(): ObfuscationSettings =
    ObfuscationSettings(
        selected_obfuscation = selectedObfuscationMode.fromDomain(),
        udp2tcp = udp2tcp.fromDomain(),
        shadowsocks = shadowsocks.fromDomain(),
    )

internal fun ModelObfuscationMode.fromDomain(): ObfuscationSettings.SelectedObfuscation =
    when (this) {
        ModelObfuscationMode.Udp2Tcp -> ObfuscationSettings.SelectedObfuscation.UDP2TCP
        ModelObfuscationMode.Shadowsocks -> ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS
        ModelObfuscationMode.Quic -> ObfuscationSettings.SelectedObfuscation.QUIC
        ModelObfuscationMode.Lwo -> ObfuscationSettings.SelectedObfuscation.LWO
        ModelObfuscationMode.Auto -> ObfuscationSettings.SelectedObfuscation.AUTO
        ModelObfuscationMode.Off -> ObfuscationSettings.SelectedObfuscation.OFF
    }

internal fun ModelUdp2TcpObfuscationSettings.fromDomain(): Udp2TcpObfuscationSettings =
    when (val port = port) {
        is Constraint.Any -> Udp2TcpObfuscationSettings()
        is Constraint.Only -> Udp2TcpObfuscationSettings(port = port.value.value)
    }

internal fun GeoLocationId.fromDomain(): GeographicLocationConstraint =
    when (val id = this@fromDomain) {
        is GeoLocationId.Country -> GeographicLocationConstraint(country = id.code)
        is GeoLocationId.City ->
            GeographicLocationConstraint(country = id.country.code, city = id.code)
        is GeoLocationId.Hostname ->
            GeographicLocationConstraint(
                country = id.country.code,
                city = id.code,
                hostname = id.code,
            )
    }

internal fun ModelCustomList.fromDomain(): CustomList =
    CustomList(id = id.value, name = name.value, locations = locations.map { it.fromDomain() })

internal fun ModelWireguardConstraints.fromDomain(): WireguardConstraints =
    WireguardConstraints(
        use_multihop = isMultihopEnabled,
        entry_location = entryLocation.fromDomain(),
        port = port.getOrNull()?.value,
        ip_version = ipVersion.getOrNull()?.fromDomain(),
    )

internal fun ModelOwnership.fromDomain(): Ownership =
    when (this) {
        ModelOwnership.MullvadOwned -> Ownership.MULLVAD_OWNED
        ModelOwnership.Rented -> Ownership.RENTED
    }

internal fun ModelRelaySettings.fromDomain(): RelaySettings =
    RelaySettings(
        normal =
            NormalRelaySettings(
                wireguard_constraints = relayConstraints.wireguardConstraints.fromDomain(),
                location = relayConstraints.location.fromDomain(),
                ownership = relayConstraints.ownership.fromDomain(),
                providers = relayConstraints.providers.fromDomain(),
            )
    )

internal fun Constraint<ModelOwnership>.fromDomain(): Ownership =
    when (this) {
        Constraint.Any -> Ownership.ANY
        is Constraint.Only -> value.fromDomain()
    }

internal fun ModelPlayPurchasePaymentToken.fromDomain(): PlayPurchasePaymentToken =
    PlayPurchasePaymentToken(value)

internal fun ModelPlayPurchase.fromDomain(): PlayPurchase =
    PlayPurchase(purchase_token = purchaseToken.fromDomain(), product_id = productId)

internal fun ModelNewAccessMethodSetting.fromDomain(): NewAccessMethodSetting =
    NewAccessMethodSetting(
        name = name.value,
        enabled = enabled,
        access_method = AccessMethod(custom = apiAccessMethod.fromDomain()),
    )

internal fun ApiAccessMethod.fromDomain(): AccessMethod =
    when (this) {
        ApiAccessMethod.Direct -> AccessMethod(direct = AccessMethod.Direct())
        ApiAccessMethod.Bridges -> AccessMethod(bridges = AccessMethod.Bridges())
        is ApiAccessMethod.CustomProxy -> AccessMethod(custom = fromDomain())
        is ApiAccessMethod.EncryptedDns ->
            AccessMethod(encrypted_dns_proxy = AccessMethod.EncryptedDnsProxy())
    }

internal fun ApiAccessMethod.CustomProxy.fromDomain(): CustomProxy =
    when (this) {
        is ApiAccessMethod.CustomProxy.Shadowsocks -> CustomProxy(shadowsocks = fromDomain())
        is ApiAccessMethod.CustomProxy.Socks5Remote -> CustomProxy(socks5remote = fromDomain())
    }

internal fun ApiAccessMethod.CustomProxy.Socks5Remote.fromDomain(): Socks5Remote =
    Socks5Remote(ip = ip, port = port.value, auth = auth?.fromDomain())

internal fun ModelSocksAuth.fromDomain(): SocksAuth =
    SocksAuth(username = username, password = password)

internal fun ApiAccessMethod.CustomProxy.Shadowsocks.fromDomain(): Shadowsocks =
    Shadowsocks(ip = ip, cipher = cipher.label, port = port.value, password = password ?: "")

internal fun ModelTransportProtocol.fromDomain(): TransportProtocol =
    when (this) {
        ModelTransportProtocol.Tcp -> TransportProtocol.TCP
        ModelTransportProtocol.Udp -> TransportProtocol.UDP
    }

internal fun ApiAccessMethodId.fromDomain(): UUID = UUID(value.toString())

internal fun ApiAccessMethodSetting.fromDomain(): AccessMethodSetting =
    AccessMethodSetting(
        name = name.value,
        id = id.fromDomain(),
        enabled = enabled,
        access_method = apiAccessMethod.fromDomain(),
    )

internal fun ModelShadowsocksSettings.fromDomain(): ShadowsocksSettings =
    when (val port = port) {
        is Constraint.Any -> ShadowsocksSettings()
        is Constraint.Only -> ShadowsocksSettings(port.value.value)
    }

internal fun ModelDaitaSettings.fromDomain(): DaitaSettings =
    DaitaSettings(enabled = enabled, direct_only = directOnly)

internal fun ModelIpVersion.fromDomain(): IpVersion =
    when (this) {
        ModelIpVersion.IPV4 -> IpVersion.V4
        ModelIpVersion.IPV6 -> IpVersion.V6
    }

internal fun ModelQuantumResistantState.fromDomain(): QuantumResistantState =
    when (this) {
        ModelQuantumResistantState.On ->
            QuantumResistantState(state = QuantumResistantState.State.ON)
        ModelQuantumResistantState.Off ->
            QuantumResistantState(state = QuantumResistantState.State.OFF)
    }
