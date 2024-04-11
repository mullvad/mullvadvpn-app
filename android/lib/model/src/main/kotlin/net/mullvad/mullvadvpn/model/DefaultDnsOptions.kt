package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@Parcelize
@optics
data class DefaultDnsOptions(
    val blockAds: Boolean = false,
    val blockTrackers: Boolean = false,
    val blockMalware: Boolean = false,
    val blockAdultContent: Boolean = false,
    val blockGambling: Boolean = false,
    val blockSocialMedia: Boolean = false,
) : Parcelable {
    fun isAnyBlockerEnabled(): Boolean {
        return blockAds ||
            blockTrackers ||
            blockMalware ||
            blockAdultContent ||
            blockGambling ||
            blockSocialMedia
    }

    companion object
}
