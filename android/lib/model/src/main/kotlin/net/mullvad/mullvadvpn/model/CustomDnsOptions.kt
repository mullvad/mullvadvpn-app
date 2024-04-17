package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Parcelize
@optics
data class CustomDnsOptions(val addresses: List<InetAddress>) : Parcelable {
    companion object
}
