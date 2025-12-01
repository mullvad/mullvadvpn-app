package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import mullvad_daemon.management_interface.AccessMethod
import mullvad_daemon.management_interface.AccessMethodSetting
import mullvad_daemon.management_interface.ApiAccessMethodSettings
import mullvad_daemon.management_interface.CustomDnsOptions
import mullvad_daemon.management_interface.CustomList
import mullvad_daemon.management_interface.CustomProxy
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
import mullvad_daemon.management_interface.Socks5Remote
import mullvad_daemon.management_interface.SocksAuth
import mullvad_daemon.management_interface.UUID
import mullvad_daemon.management_interface.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod as ModelApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId as ModelApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions as ModelCustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions as ModelDefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion as ModelIpVersion
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting as ModelNewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode as ModelObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings as ModelObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.lib.model.PlayPurchase as ModelPlayPurchase
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken as ModelPlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelaySettings as ModelRelaySettings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksObfuscationSettings as ModelShadowsocksObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.SocksAuth as ModelSocksAuth
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints as ModelWireguardConstraints
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting as ModelApiAccessMethodSetting

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
        wireguard_port = wireguardPort.fromDomain(),
    )

internal fun ModelObfuscationMode.fromDomain(): ObfuscationSettings.SelectedObfuscation =
    when (this) {
        ObfuscationMode.Udp2Tcp -> ObfuscationSettings.SelectedObfuscation.UDP2TCP
        ObfuscationMode.Shadowsocks -> ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS
        ObfuscationMode.WireguardPort -> ObfuscationSettings.SelectedObfuscation.WIREGUARD_PORT
        ObfuscationMode.Quic -> ObfuscationSettings.SelectedObfuscation.QUIC
        ObfuscationMode.Lwo -> ObfuscationSettings.SelectedObfuscation.LWO
        ObfuscationMode.Auto -> ObfuscationSettings.SelectedObfuscation.AUTO
        ObfuscationMode.Off -> ObfuscationSettings.SelectedObfuscation.OFF
    }

internal fun Udp2TcpObfuscationSettings.fromDomain(): ObfuscationSettings.Udp2TcpObfuscation =
    when (val port = port) {
        is Constraint.Any -> ObfuscationSettings.Udp2TcpObfuscation(port = null)
        is Constraint.Only -> ObfuscationSettings.Udp2TcpObfuscation(port.value.value)
    }

internal fun Constraint<Port>.fromDomain(): ObfuscationSettings.WireguardPort =
    when (this) {
        is Constraint.Any -> ObfuscationSettings.WireguardPort(port = null)
        is Constraint.Only -> ObfuscationSettings.WireguardPort(this.value.value)
    }

internal fun GeoLocationId.fromDomain(): GeographicLocationConstraint =
    when (val id = this) {
        is GeoLocationId.Country -> GeographicLocationConstraint(id.code)
        is GeoLocationId.City ->
            GeographicLocationConstraint(country = id.country.code, city = id.code)
        is GeoLocationId.Hostname ->
            GeographicLocationConstraint(
                country = id.country.code,
                city = id.city.code,
                hostname = id.code,
            )
    }

internal fun ModelWireguardConstraints.fromDomain(): WireguardConstraints =
    WireguardConstraints(
        use_multihop = isMultihopEnabled,
        entry_location = entryLocation.fromDomain(),
        ip_version = ipVersion.getOrNull()?.fromDomain(),
    )

internal fun ModelCustomList.fromDomain(): CustomList =
    CustomList(id = id.value, name = name.value, locations = locations.map { it.fromDomain() })

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

internal fun ModelApiAccessMethodSetting.fromDomain(): AccessMethodSetting =
    AccessMethodSetting(
        id = id.fromDomain(),
        name = name.value,
        enabled = enabled,
        access_method = apiAccessMethod.fromDomain(),
    )

internal fun ModelApiAccessMethod.fromDomain(): AccessMethod =
    when (this) {
        ModelApiAccessMethod.Direct -> AccessMethod(direct = AccessMethod.Direct())
        ModelApiAccessMethod.Bridges -> AccessMethod(bridges = AccessMethod.Bridges())
        ModelApiAccessMethod.EncryptedDns ->
            AccessMethod(encrypted_dns_proxy = AccessMethod.EncryptedDnsProxy())
        is ModelApiAccessMethod.CustomProxy -> AccessMethod(custom = fromDomain())
    }

internal fun ModelApiAccessMethod.CustomProxy.fromDomain(): CustomProxy =
    when (this) {
        is ModelApiAccessMethod.CustomProxy.Shadowsocks -> CustomProxy(shadowsocks = fromDomain())
        is ModelApiAccessMethod.CustomProxy.Socks5Remote -> CustomProxy(socks5remote = fromDomain())
    }

internal fun ModelSocksAuth.fromDomain(): SocksAuth =
    SocksAuth(username = username, password = password)

internal fun ModelApiAccessMethod.CustomProxy.Shadowsocks.fromDomain(): Shadowsocks =
    Shadowsocks(ip = ip, cipher = cipher.label, port = port.value, password = password ?: "")

internal fun ModelApiAccessMethodId.fromDomain(): UUID = UUID(value.toString())

internal fun ModelApiAccessMethod.CustomProxy.Socks5Remote.fromDomain(): Socks5Remote =
    Socks5Remote(ip = ip, port = port.value, auth = auth?.fromDomain())

internal fun ModelShadowsocksObfuscationSettings.fromDomain(): ObfuscationSettings.Shadowsocks =
    when (val port = port) {
        is Constraint.Any -> ObfuscationSettings.Shadowsocks(port = null)
        is Constraint.Only -> ObfuscationSettings.Shadowsocks(port = port.value.value)
    }

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
