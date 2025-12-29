package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics

@optics
data class DefaultDnsOptions(
    val blockAds: Boolean = false,
    val blockTrackers: Boolean = false,
    val blockMalware: Boolean = false,
    val blockAdultContent: Boolean = false,
    val blockGambling: Boolean = false,
    val blockSocialMedia: Boolean = false,
) {
    val isAnyBlockerEnabled: Boolean =
        blockAds ||
            blockTrackers ||
            blockMalware ||
            blockAdultContent ||
            blockGambling ||
            blockSocialMedia

    val isAllBlockersEnabled: Boolean =
        blockAds &&
            blockTrackers &&
            blockMalware &&
            blockAdultContent &&
            blockGambling &&
            blockSocialMedia

    fun numberOfBlockersEnabled(): Int {
        var blocked = 0
        if (blockAds) blocked++
        if (blockTrackers) blocked++
        if (blockMalware) blocked++
        if (blockAdultContent) blocked++
        if (blockGambling) blocked++
        if (blockSocialMedia) blocked++
        return blocked
    }

    companion object
}
