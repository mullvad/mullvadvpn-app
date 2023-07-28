package net.mullvad.mullvadvpn.compose.state

import org.joda.time.DateTime

data class AccountUiState(
    val deviceName: String,
    val accountNumber: String,
    val accountExpiry: DateTime?
)
