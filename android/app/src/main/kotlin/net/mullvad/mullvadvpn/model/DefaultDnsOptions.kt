package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class DefaultDnsOptions(
    val blockAds: Boolean = false,
    val blockTrackers: Boolean = false,
    val blockMalware: Boolean = false,
    val blockAdultContent: Boolean = false,
    val blockGambling: Boolean = false,
) : Parcelable
