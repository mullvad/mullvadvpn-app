package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.talpid.net.wireguard.TunnelOptions as TalpidWireguardTunnelOptions

@Parcelize
data class WireguardTunnelOptions(val options: TalpidWireguardTunnelOptions) : Parcelable
