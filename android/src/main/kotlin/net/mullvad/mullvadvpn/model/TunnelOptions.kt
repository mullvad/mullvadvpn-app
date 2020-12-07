package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.talpid.net.wireguard.TunnelOptions as WireguardTunnelOptions

@Parcelize
data class TunnelOptions(
    val wireguard: WireguardTunnelOptions,
    val dnsOptions: DnsOptions
) : Parcelable
