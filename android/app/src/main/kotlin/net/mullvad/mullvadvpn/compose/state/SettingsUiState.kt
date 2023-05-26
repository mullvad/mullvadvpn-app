package net.mullvad.mullvadvpn.compose.state

import org.joda.time.DateTime

data class SettingsUiState(
    val isLoggedIn: Boolean,
    val accountExpiry: DateTime?,
    val appVersion: String,
    val isUpdateAvailable: Boolean
)
