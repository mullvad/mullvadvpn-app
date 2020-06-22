package net.mullvad.mullvadvpn.model

data class AppVersionInfo(
    val supported: Boolean,
    val suggestedUpgrade: String?
)
