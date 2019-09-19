package net.mullvad.mullvadvpn.model

data class AppVersionInfo(
    val currentIsSupported: Boolean,
    val currentIsOutdated: Boolean,
    val latestStable: String,
    val latest: String
)
