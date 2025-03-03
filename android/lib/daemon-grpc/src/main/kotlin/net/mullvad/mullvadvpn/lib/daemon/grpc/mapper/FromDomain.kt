package net.mullvad.mullvadvpn.lib.daemon.grpc.mapper

import mullvad_daemon.management_interface.ManagementInterface
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.DaitaSettings
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.NewAccessMethodSetting
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.PlayPurchase
import net.mullvad.mullvadvpn.lib.model.PlayPurchasePaymentToken
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelaySettings
import net.mullvad.mullvadvpn.lib.model.ShadowsocksSettings
import net.mullvad.mullvadvpn.lib.model.SocksAuth
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
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
        is Constraint.Only -> value.map { it.value }
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
        .setSelectedObfuscation(selectedObfuscationMode.fromDomain())
        .setUdp2Tcp(udp2tcp.fromDomain())
        .setShadowsocks(shadowsocks.fromDomain())
        .build()

internal fun ObfuscationMode.fromDomain():
    ManagementInterface.ObfuscationSettings.SelectedObfuscation =
    when (this) {
        ObfuscationMode.Udp2Tcp ->
            ManagementInterface.ObfuscationSettings.SelectedObfuscation.UDP2TCP
        ObfuscationMode.Shadowsocks ->
            ManagementInterface.ObfuscationSettings.SelectedObfuscation.SHADOWSOCKS
        ObfuscationMode.Auto -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.AUTO
        ObfuscationMode.Off -> ManagementInterface.ObfuscationSettings.SelectedObfuscation.OFF
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
                is GeoLocationId.Country -> setCountry(id.code)
                is GeoLocationId.City -> setCountry(id.country.code).setCity(id.code)
                is GeoLocationId.Hostname ->
                    setCountry(id.country.code).setCity(id.city.code).setHostname(id.code)
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
    ManagementInterface.WireguardConstraints.newBuilder()
        .setUseMultihop(isMultihopEnabled)
        .setEntryLocation(entryLocation.fromDomain())
        .apply {
            when (val port = this@fromDomain.port) {
                is Constraint.Any -> clearPort()
                is Constraint.Only -> setPort(port.value.value)
            }
        }
        .apply {
            when (val ipVersion = this@fromDomain.ipVersion) {
                is Constraint.Any -> clearIpVersion()
                is Constraint.Only -> setIpVersion(ipVersion.value.fromDomain())
            }
        }
        .build()

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

internal fun NewAccessMethodSetting.fromDomain(): ManagementInterface.NewAccessMethodSetting =
    ManagementInterface.NewAccessMethodSetting.newBuilder()
        .setName(name.value)
        .setEnabled(enabled)
        .setAccessMethod(
            ManagementInterface.AccessMethod.newBuilder().setCustom(apiAccessMethod.fromDomain())
        )
        .build()

internal fun ApiAccessMethod.fromDomain(): ManagementInterface.AccessMethod =
    ManagementInterface.AccessMethod.newBuilder()
        .let {
            when (this) {
                ApiAccessMethod.Direct ->
                    it.setDirect(ManagementInterface.AccessMethod.Direct.getDefaultInstance())
                ApiAccessMethod.Bridges ->
                    it.setBridges(ManagementInterface.AccessMethod.Bridges.getDefaultInstance())
                is ApiAccessMethod.CustomProxy -> it.setCustom(fromDomain())
                is ApiAccessMethod.EncryptedDns ->
                    it.setEncryptedDnsProxy(
                        ManagementInterface.AccessMethod.EncryptedDnsProxy.getDefaultInstance()
                    )
            }
        }
        .build()

internal fun ApiAccessMethod.CustomProxy.fromDomain(): ManagementInterface.CustomProxy =
    ManagementInterface.CustomProxy.newBuilder()
        .let {
            when (this) {
                is ApiAccessMethod.CustomProxy.Shadowsocks -> it.setShadowsocks(fromDomain())
                is ApiAccessMethod.CustomProxy.Socks5Remote -> it.setSocks5Remote(fromDomain())
            }
        }
        .build()

internal fun ApiAccessMethod.CustomProxy.Socks5Remote.fromDomain():
    ManagementInterface.Socks5Remote =
    ManagementInterface.Socks5Remote.newBuilder().setIp(ip).setPort(port.value).let {
        auth?.let { auth -> it.setAuth(auth.fromDomain()) }
        it.build()
    }

internal fun SocksAuth.fromDomain(): ManagementInterface.SocksAuth =
    ManagementInterface.SocksAuth.newBuilder().setUsername(username).setPassword(password).build()

internal fun ApiAccessMethod.CustomProxy.Shadowsocks.fromDomain(): ManagementInterface.Shadowsocks =
    ManagementInterface.Shadowsocks.newBuilder()
        .setIp(ip)
        .setCipher(cipher.label)
        .setPort(port.value)
        .let {
            if (password != null) {
                it.setPassword(password)
            }
            it.build()
        }

internal fun TransportProtocol.fromDomain(): ManagementInterface.TransportProtocol =
    when (this) {
        TransportProtocol.Tcp -> ManagementInterface.TransportProtocol.TCP
        TransportProtocol.Udp -> ManagementInterface.TransportProtocol.UDP
    }

internal fun ApiAccessMethodId.fromDomain(): ManagementInterface.UUID =
    ManagementInterface.UUID.newBuilder().setValue(value.toString()).build()

internal fun ApiAccessMethodSetting.fromDomain(): ManagementInterface.AccessMethodSetting =
    ManagementInterface.AccessMethodSetting.newBuilder()
        .setName(name.value)
        .setId(id.fromDomain())
        .setEnabled(enabled)
        .setAccessMethod(apiAccessMethod.fromDomain())
        .build()

internal fun ShadowsocksSettings.fromDomain(): ManagementInterface.ShadowsocksSettings =
    when (val port = port) {
        is Constraint.Any ->
            ManagementInterface.ShadowsocksSettings.newBuilder().clearPort().build()
        is Constraint.Only ->
            ManagementInterface.ShadowsocksSettings.newBuilder().setPort(port.value.value).build()
    }

internal fun DaitaSettings.fromDomain(): ManagementInterface.DaitaSettings =
    ManagementInterface.DaitaSettings.newBuilder()
        .setEnabled(enabled)
        .setDirectOnly(directOnly)
        .build()

internal fun IpVersion.fromDomain(): ManagementInterface.IpVersion =
    when (this) {
        IpVersion.IPV4 -> ManagementInterface.IpVersion.V4
        IpVersion.IPV6 -> ManagementInterface.IpVersion.V6
    }
