package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
@JvmInline
value class ProviderId(val value: String) : Comparable<ProviderId>, Parcelable {
    override fun compareTo(other: ProviderId): Int =
        value.uppercase().compareTo(other.value.uppercase())
}
