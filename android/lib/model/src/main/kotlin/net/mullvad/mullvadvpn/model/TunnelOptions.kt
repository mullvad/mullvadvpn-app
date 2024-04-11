package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@Parcelize
@optics
data class TunnelOptions(val wireguard: WireguardTunnelOptions, val dnsOptions: DnsOptions) :
    Parcelable {
    companion object
}
