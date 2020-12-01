package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Parcelize
data class GeoIpLocation(
    val ipv4: InetAddress?,
    val ipv6: InetAddress?,
    val country: String,
    val city: String?,
    val hostname: String?
) : Parcelable
