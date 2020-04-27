package net.mullvad.mullvadvpn.model

data class AppVersionInfo(
    val supported: Boolean,
    val latestStable: String,
    val latestBeta: String,
    val latest: String
)
