@file:Suppress("TooManyFunctions")

package net.mullvad.mullvadvpn.lib.grpc.mapper

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
import mullvad_daemon.management_interface.Socks5Remote
import mullvad_daemon.management_interface.SocksAuth
import mullvad_daemon.management_interface.UUID
import mullvad_daemon.management_interface.WireguardConstraints
import mullvad_daemon.relay_selector.EntryConstraints
import mullvad_daemon.relay_selector.ExitConstraints
import mullvad_daemon.relay_selector.MultiHopConstraints
import mullvad_daemon.relay_selector.Predicate
import mullvad_daemon.relay_selector.Provider
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod.CustomProxy as ModelCustomProxy
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.Cipher as ModelCipher
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions as ModelCustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList as ModelCustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.DaitaSettings as ModelDaitaSettings
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions as ModelDefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions as ModelDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState as ModelDnsState
import net.mullvad.mullvadvpn.lib.model.EntryConstraints as ModelEntryConstraints
import net.mullvad.mullvadvpn.lib.model.ExitConstraints as ModelExitConstraints
import net.mullvad.mullvadvpn.lib.model.GeoLocationId as ModelGeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion as ModelIpVersion
import net.mullvad.mullvadvpn.lib.model.LwoObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.MultihopConstraints as ModelMultihopConstraints
import net.mullvad.mullvadvpn.lib.model.MultihopMode as ModelMultihopMode
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting as ModelNewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode as ModelObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings as ModelObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Ownership as ModelOwnership
import net.mullvad.mullvadvpn.lib.model.PlayPurchase as ModelPlayPurchase
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken as ModelPlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState as ModelQuantumResistantState
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelaySelectorPredicate
import net.mullvad.mullvadvpn.lib.model.RelaySettings as ModelRelaySettings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksObfuscationSettings as ModelShadowsocksObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.SocksAuth as ModelSocksAuth
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings as ModelUdp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints as ModelWireguardConstraints

internal fun Constraint<RelayItemId>.fromDomain(): LocationConstraint =
    when (this) {
        Constraint.Any -> LocationConstraint()
        is Constraint.Only ->
            when (val relayItemId = this@fromDomain.value) {
                is CustomListId -> LocationConstraint(custom_list = relayItemId.value)
                is ModelGeoLocationId -> LocationConstraint(location = relayItemId.fromDomain())
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
        lwo = lwo.fromDomain(),
    )

internal fun ModelObfuscationMode.fromDomain(): ObfuscationSettings.SelectedObfuscation =
    when (this) {
        ModelObfuscationMode.Udp2Tcp -> ObfuscationSettings.SelectedObfuscation.UDP2TCP
        ModelObfuscationMode.Shadowsocks -> ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS
        ModelObfuscationMode.WireguardPort -> ObfuscationSettings.SelectedObfuscation.WIREGUARD_PORT
        ModelObfuscationMode.Quic -> ObfuscationSettings.SelectedObfuscation.QUIC
        ModelObfuscationMode.Lwo -> ObfuscationSettings.SelectedObfuscation.LWO
        ModelObfuscationMode.Auto -> ObfuscationSettings.SelectedObfuscation.AUTO
        ModelObfuscationMode.Off -> ObfuscationSettings.SelectedObfuscation.OFF
    }

internal fun ModelUdp2TcpObfuscationSettings.fromDomain(): ObfuscationSettings.Udp2TcpObfuscation =
    when (val port = port) {
        is Constraint.Any -> ObfuscationSettings.Udp2TcpObfuscation()
        is Constraint.Only -> ObfuscationSettings.Udp2TcpObfuscation(port = port.value.value)
    }

internal fun Constraint<Port>.fromDomain(): ObfuscationSettings.WireguardPort =
    when (this) {
        is Constraint.Any -> ObfuscationSettings.WireguardPort()
        is Constraint.Only -> ObfuscationSettings.WireguardPort(port = value.value)
    }

internal fun ModelGeoLocationId.fromDomain(): GeographicLocationConstraint =
    when (this) {
        is ModelGeoLocationId.Country -> GeographicLocationConstraint(country = code)
        is ModelGeoLocationId.City ->
            GeographicLocationConstraint(country = country.code, city = code)
        is ModelGeoLocationId.Hostname ->
            GeographicLocationConstraint(country = country.code, city = city.code, hostname = code)
    }

internal fun ModelCustomList.fromDomain(): CustomList =
    CustomList(id = id.value, name = name.value, locations = locations.map { it.fromDomain() })

internal fun ModelWireguardConstraints.fromDomain(): WireguardConstraints =
    WireguardConstraints(
        multihop = multihop.fromDomain(),
        entry_location = entryLocation.fromDomain(),
        entry_ownership = entryOwnership.fromDomain(),
        entry_providers = entryProviders.fromDomain(),
        ip_version =
            when (val ipVersion = ipVersion) {
                is Constraint.Any -> null
                is Constraint.Only -> ipVersion.value.fromDomain()
            },
    )

internal fun ModelMultihopMode.fromDomain(): WireguardConstraints.Multihop =
    when (this) {
        ModelMultihopMode.WHEN_NEEDED -> WireguardConstraints.Multihop.Auto
        ModelMultihopMode.ALWAYS -> WireguardConstraints.Multihop.Always
        ModelMultihopMode.NEVER -> WireguardConstraints.Multihop.Never
    }

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

internal fun ModelCustomProxy.fromDomain(): CustomProxy =
    when (this) {
        is ModelCustomProxy.Shadowsocks -> CustomProxy(shadowsocks = fromDomain())
        is ModelCustomProxy.Socks5Remote -> CustomProxy(socks5remote = fromDomain())
    }

internal fun ModelCustomProxy.Socks5Remote.fromDomain(): Socks5Remote =
    Socks5Remote(
        ip = ip,
        port = port.value,
        auth = auth?.fromDomain(),
    )

internal fun ModelSocksAuth.fromDomain(): SocksAuth =
    SocksAuth(username = username, password = password)

internal fun ModelCustomProxy.Shadowsocks.fromDomain(): Shadowsocks =
    Shadowsocks(
        ip = ip,
        cipher = cipher.fromDomain(),
        port = port.value,
        password = password ?: "",
    )

internal fun ApiAccessMethodId.fromDomain(): UUID = UUID(value = value.toString())

internal fun ApiAccessMethodSetting.fromDomain(): AccessMethodSetting =
    AccessMethodSetting(
        id = id.fromDomain(),
        name = name.value,
        enabled = enabled,
        access_method = apiAccessMethod.fromDomain(),
    )

internal fun ApiAccessMethod.fromDomain(): AccessMethod =
    when (this) {
        ApiAccessMethod.Bridges -> AccessMethod(bridges = AccessMethod.Bridges())
        is ModelCustomProxy.Shadowsocks ->
            AccessMethod(custom = CustomProxy(shadowsocks = fromDomain()))
        is ModelCustomProxy.Socks5Remote ->
            AccessMethod(custom = CustomProxy(socks5remote = fromDomain()))
        ApiAccessMethod.Direct -> AccessMethod(direct = AccessMethod.Direct())
        ApiAccessMethod.EncryptedDns ->
            AccessMethod(encrypted_dns_proxy = AccessMethod.EncryptedDnsProxy())
    }

internal fun ModelShadowsocksObfuscationSettings.fromDomain(): ObfuscationSettings.Shadowsocks =
    when (val port = port) {
        is Constraint.Any -> ObfuscationSettings.Shadowsocks()
        is Constraint.Only -> ObfuscationSettings.Shadowsocks(port = port.value.value)
    }

internal fun LwoObfuscationSettings.fromDomain(): ObfuscationSettings.Lwo =
    when (val port = port) {
        is Constraint.Any -> ObfuscationSettings.Lwo()
        is Constraint.Only -> ObfuscationSettings.Lwo(port = port.value.value)
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

internal fun RelaySelectorPredicate.fromDomain(): Predicate =
    when (this) {
        is RelaySelectorPredicate.Autohop -> fromDomain()
        is RelaySelectorPredicate.Entry -> fromDomain()
        is RelaySelectorPredicate.Exit -> fromDomain()
        is RelaySelectorPredicate.SingleHop -> fromDomain()
    }

internal fun RelaySelectorPredicate.SingleHop.fromDomain() =
    Predicate(singlehop = entryConstraints.fromDomain())

internal fun RelaySelectorPredicate.Autohop.fromDomain() =
    Predicate(autohop = entryConstraints.fromDomain())

internal fun RelaySelectorPredicate.Entry.fromDomain() =
    Predicate(entry = multihopConstraints.fromDomain())

internal fun RelaySelectorPredicate.Exit.fromDomain() =
    Predicate(exit = multihopConstraints.fromDomain())

internal fun ModelMultihopConstraints.fromDomain(): MultiHopConstraints =
    MultiHopConstraints(
        entry = entryConstraints.fromDomain(),
        exit = exitConstraints.fromDomain(),
    )

internal fun ModelEntryConstraints.fromDomain(): EntryConstraints =
    EntryConstraints(
        general_constraints = generalConstraints.fromDomain(),
        obfuscation_settings = obfuscation.getOrNull()?.fromDomain(),
        daita_settings = daitaSettings.getOrNull()?.fromDomain(),
        ip_version = ipVersion.getOrNull()?.fromDomain(),
    )

internal fun ModelExitConstraints.fromDomain(): ExitConstraints =
    ExitConstraints(
        location = location.getOrNull()?.fromDomain(),
        providers = providers.fromDomain().map { Provider(name = it) },
        ownership = ownership.getOrNull()?.fromDomain() ?: Ownership.ANY,
    )

internal fun ModelDaitaSettings.fromDomain(): DaitaSettings = DaitaSettings(enabled = enabled)

internal fun RelayItemId.fromDomain(): LocationConstraint =
    when (this) {
        is CustomListId -> LocationConstraint(custom_list = value)
        is ModelGeoLocationId -> LocationConstraint(location = fromDomain())
    }

internal fun ModelCipher.fromDomain(): Shadowsocks.Cipher = Shadowsocks.Cipher(name = value)
