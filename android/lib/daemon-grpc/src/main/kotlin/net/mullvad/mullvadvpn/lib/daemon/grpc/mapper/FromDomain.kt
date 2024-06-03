package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.PlayPurchase
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelaySettings
import net.mullvad.mullvadvpn.lib.model.SelectedObfuscation
import net.mullvad.mullvadvpn.lib.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints

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

internal fun Constraint<Providers>.fromDomain(): List<String> =
    when (this) {
        is Constraint.Any -> emptyList()
        is Constraint.Only -> value.providers.map { it.value }
    }

internal fun DnsOptions.fromDomain(): ManagementInterface.DnsOptions =
    ManagementInterface.DnsOptions.newBuilder()
        .setState(state.fromDomain())
        .setCustomOptions(customOptions.fromDomain())
        .setDefaultOptions(defaultOptions.fromDomain())
        .build()

internal fun DnsState.fromDomain(): ManagementInterface.DnsOptions.DnsState =
    when (this) {
        DnsState.Default -> ManagementInterface.DnsOptions.DnsState.DEFAULT
        DnsState.Custom -> ManagementInterface.DnsOptions.DnsState.CUSTOM
    }

internal fun CustomDnsOptions.fromDomain(): ManagementInterface.CustomDnsOptions =
    ManagementInterface.CustomDnsOptions.newBuilder()
        .addAllAddresses(addresses.map { it.hostAddress })
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

internal fun ObfuscationSettings.fromDomain(): ManagementInterface.ObfuscationSettings =
    ManagementInterface.ObfuscationSettings.newBuilder()
        .setSelectedObfuscation(selectedObfuscation.fromDomain())
        .setUdp2Tcp(udp2tcp.fromDomain())
        .build()

internal fun SelectedObfuscation.fromDomain():
    ManagementInterface.ObfuscationSettings.SelectedObfuscation =
    when (this) {
        SelectedObfuscation.Udp2Tcp ->
            ManagementInterface.ObfuscationSettings.SelectedObfuscation.UDP2TCP
        SelectedObfuscation.Auto -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.AUTO
        SelectedObfuscation.Off -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.OFF
    }

internal fun Udp2TcpObfuscationSettings.fromDomain():
    ManagementInterface.Udp2TcpObfuscationSettings =
    when (val port = port) {
        is Constraint.Any ->
            ManagementInterface.Udp2TcpObfuscationSettings.newBuilder().clearPort().build()
        is Constraint.Only ->
            ManagementInterface.Udp2TcpObfuscationSettings.newBuilder()
                .setPort(port.value.value)
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
        .setId(id.value)
        .setName(name.value)
        .addAllLocations(locations.map { it.fromDomain() })
        .build()

internal fun WireguardConstraints.fromDomain(): ManagementInterface.WireguardConstraints =
    when (port) {
        is Constraint.Any -> ManagementInterface.WireguardConstraints.newBuilder().build()
        is Constraint.Only ->
            ManagementInterface.WireguardConstraints.newBuilder()
                .setPort((port as Constraint.Only<Port>).value.value)
                .build()
    }

internal fun Ownership.fromDomain(): ManagementInterface.Ownership =
    when (this) {
        Ownership.MullvadOwned -> ManagementInterface.Ownership.MULLVAD_OWNED
        Ownership.Rented -> ManagementInterface.Ownership.RENTED
    }

internal fun RelaySettings.fromDomain(): ManagementInterface.RelaySettings =
    ManagementInterface.RelaySettings.newBuilder()
        .setNormal(
            ManagementInterface.NormalRelaySettings.newBuilder()
                .setTunnelType(ManagementInterface.TunnelType.WIREGUARD)
                .setWireguardConstraints(relayConstraints.wireguardConstraints.fromDomain())
                .setOpenvpnConstraints(ManagementInterface.OpenvpnConstraints.getDefaultInstance())
                .setLocation(relayConstraints.location.fromDomain())
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

internal fun PlayPurchasePaymentToken.fromDomain(): ManagementInterface.PlayPurchasePaymentToken =
    ManagementInterface.PlayPurchasePaymentToken.newBuilder().setToken(value).build()

internal fun PlayPurchase.fromDomain(): ManagementInterface.PlayPurchase =
    ManagementInterface.PlayPurchase.newBuilder()
        .setPurchaseToken(purchaseToken.fromDomain())
        .setProductId(productId)
        .build()
