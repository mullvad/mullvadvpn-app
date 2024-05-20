package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayItemId
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.WireguardConstraints

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
        .setSelectedObfuscation(selectedObfuscation.toDomain())
        .setUdp2Tcp(udp2tcp.toDomain())
        .build()

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
