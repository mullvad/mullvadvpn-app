package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Parcelize
data class RelayOverride(
    val hostname: String,
    val ipv4AddressIn: InetAddress?,
    val ipv6AddressIn: InetAddress?
) : Parcelable
