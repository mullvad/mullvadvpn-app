package net.mullvad.mullvadvpn.model

data class AppVersionInfo(
    val supported: Boolean,
    val latest: String,
    val latestStable: String,
    val latestBeta: String
)
