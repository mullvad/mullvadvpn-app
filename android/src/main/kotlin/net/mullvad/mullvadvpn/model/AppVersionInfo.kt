package net.mullvad.mullvadvpn.model

data class AppVersionInfo(
    val currentIsSupported: Boolean,
    val latestStable: String,
    val latest: String
)
